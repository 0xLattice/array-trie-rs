//! Path-segment trie for declarative traversal, transforms, and resolution plans.
use std::collections::BTreeMap;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;
fn eq(a: &str, b: &str) -> bool {
    a.nfc().eq(b.nfc())
}
fn strip<'a>(s: &'a str, p: &str) -> Option<&'a str> {
    let mut chars = s.graphemes(true);
    let mut offset = 0;
    for c in p.graphemes(true) {
        let n = chars.next()?;
        if !eq(c, n) {
            return None;
        }
        offset += n.len();
    }
    Some(&s[offset..])
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArrayTrie<V> {
    pub value: Option<V>,
    pub children: BTreeMap<String, ArrayTrie<V>>,
}
impl<V> Default for ArrayTrie<V> {
    fn default() -> Self {
        Self {
            value: None,
            children: BTreeMap::new(),
        }
    }
}
impl<V> ArrayTrie<V> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn is_empty(&self) -> bool {
        self.value.is_none() && self.children.is_empty()
    }
    pub fn get<S: AsRef<str>>(&self, path: &[S]) -> Option<&V> {
        self.traverse(path)?.value.as_ref()
    }
    pub fn set<S: AsRef<str>>(&mut self, path: &[S], value: V) {
        if let Some((head, tail)) = path.split_first() {
            let key = self
                .children
                .keys()
                .find(|k| eq(k, head.as_ref()))
                .cloned()
                .unwrap_or_else(|| head.as_ref().into());
            self.children.entry(key).or_default().set(tail, value)
        } else {
            self.value = Some(value)
        }
    }
    pub fn traverse<S: AsRef<str>>(&self, path: &[S]) -> Option<&Self> {
        if let Some((head, tail)) = path.split_first() {
            self.children
                .iter()
                .find(|(k, _)| eq(k, head.as_ref()))?
                .1
                .traverse(tail)
        } else {
            Some(self)
        }
    }
    pub fn remove<S: AsRef<str>>(&mut self, path: &[S]) -> Option<V> {
        if let Some((head, tail)) = path.split_first() {
            let key = self.children.keys().find(|k| eq(k, head.as_ref()))?.clone();
            let child = self.children.get_mut(&key)?;
            let old = child.remove(tail);
            if child.is_empty() {
                self.children.remove(&key);
            }
            old
        } else {
            self.value.take()
        }
    }
    pub fn child_values(&self) -> Vec<&V> {
        self.children
            .values()
            .filter_map(|t| t.value.as_ref())
            .collect()
    }
    pub fn child_keys(&self) -> Vec<&str> {
        self.children.keys().map(String::as_str).collect()
    }
    pub fn child_characters(&self) -> Vec<&str> {
        let mut chars: Vec<_> = self
            .children
            .keys()
            .filter_map(|k| k.graphemes(true).next())
            .collect();
        chars.sort_by(|a, b| a.nfc().cmp(b.nfc()));
        chars.dedup_by(|a, b| eq(a, b));
        chars
    }
    pub fn all_values(&self) -> Vec<&V> {
        let mut values = vec![];
        if let Some(v) = &self.value {
            values.push(v)
        }
        for child in self.children.values() {
            values.extend(child.all_values())
        }
        values
    }
    pub fn values_excluding_prefix(&self, prefix: &str) -> Vec<&V> {
        if prefix.is_empty() {
            return vec![];
        }
        let mut values: Vec<_> = self.value.iter().collect();
        values.extend(
            self.children
                .iter()
                .filter(|(k, _)| strip(k, prefix).is_none())
                .flat_map(|(_, t)| t.all_values()),
        );
        values
    }
    pub fn child_prefix(&self, c: &str) -> Option<String> {
        let keys: Vec<_> = self
            .children
            .keys()
            .filter(|k| k.graphemes(true).next().is_some_and(|first| eq(first, c)))
            .collect();
        let first = keys.first()?;
        let mut len = first.len();
        for k in &keys[1..] {
            len = first
                .graphemes(true)
                .zip(k.graphemes(true))
                .take_while(|(a, b)| eq(a, b))
                .map(|(g, _)| g.len())
                .sum::<usize>()
                .min(len);
        }
        Some(first[..len].to_string())
    }
}
impl<V: Clone> ArrayTrie<V> {
    pub fn values_along_path(&self, path: &str) -> Vec<(Self, &V)> {
        if path.is_empty() {
            return vec![];
        }
        self.children
            .iter()
            .filter(|(k, _)| strip(path, k).is_some())
            .filter_map(|(_, t)| {
                t.value.as_ref().map(|v| {
                    let mut subtree = t.clone();
                    subtree.value = None;
                    (subtree, v)
                })
            })
            .collect()
    }

    pub fn deleting<S: AsRef<str>>(&self, path: &[S]) -> Self {
        let mut out = self.clone();
        out.remove(path);
        out
    }
    pub fn traverse_child(&self, c: &str) -> Option<Self> {
        let children: BTreeMap<_, _> = self
            .children
            .iter()
            .filter(|(k, _)| k.graphemes(true).next().is_some_and(|first| eq(first, c)))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        if children.is_empty() {
            None
        } else {
            Some(Self {
                value: self.value.clone(),
                children,
            })
        }
    }
    /// Traverse within the first segment; remaining segment suffixes remain child keys.
    pub fn traverse_prefix(&self, prefix: &str) -> Option<Self> {
        let children: BTreeMap<_, _> = self
            .children
            .iter()
            .filter_map(|(k, v)| strip(k, prefix).map(|rest| (rest.to_string(), v.clone())))
            .collect();
        if children.is_empty() {
            None
        } else {
            Some(Self {
                value: self
                    .children
                    .iter()
                    .find(|(k, _)| eq(k, prefix))
                    .and_then(|(_, t)| t.value.clone()),
                children,
            })
        }
    }
    pub fn merging(&self, other: &Self, merge: impl Fn(&V, &V) -> V + Copy) -> Self {
        let mut out = self.clone();
        out.value = match (&self.value, &other.value) {
            (Some(a), Some(b)) => Some(merge(a, b)),
            (None, Some(b)) => Some(b.clone()),
            (a, _) => a.clone(),
        };
        for (k, v) in &other.children {
            let original = self
                .children
                .keys()
                .find(|key| eq(key, k))
                .cloned()
                .unwrap_or_else(|| k.clone());
            out.children.insert(
                original.clone(),
                self.children
                    .get(&original)
                    .map(|a| a.merging(v, merge))
                    .unwrap_or_else(|| v.clone()),
            );
        }
        out
    }
    pub fn merge_all(tries: &[Self], merge: impl Fn(&V, &V) -> V + Copy) -> Self {
        tries
            .iter()
            .fold(Self::new(), |out, t| out.merging(t, merge))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn segments_and_prefixes() {
        let mut t = ArrayTrie::new();
        t.set(&["users", "alice"], 1);
        t.set(&["users"], 2);
        t.set::<&str>(&[], 3);
        t.set(&["usage"], 4);
        assert_eq!(t.get(&["users", "alice"]), Some(&1));
        assert_eq!(t.traverse(&["users"]).unwrap().get(&["alice"]), Some(&1));
        assert_eq!(t.child_prefix("u"), Some("us".into()));
        let old = t.clone();
        assert_eq!(t.remove(&["users"]), Some(2));
        assert_eq!(t.get(&["users", "alice"]), Some(&1));
        assert_eq!(old.get(&["users"]), Some(&2));
    }
    #[test]
    fn merge_and_delete() {
        let mut a = ArrayTrie::new();
        a.set(&["a"], 1);
        let mut b = ArrayTrie::new();
        b.set(&["a"], 2);
        b.set(&["a", "b"], 4);
        let c = a.merging(&b, |a, b| a + b);
        assert_eq!(c.get(&["a"]), Some(&3));
        assert_eq!(c.deleting(&["a", "b"]).get(&["a"]), Some(&3));
    }
}
