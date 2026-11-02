use lattice_array_trie::ArrayTrie;
use std::collections::{BTreeMap, BTreeSet};
fn paths() -> Vec<Vec<&'static str>> {
    vec![
        vec![],
        vec!["a"],
        vec!["ab"],
        vec!["b"],
        vec!["c"],
        vec!["api", "v1"],
        vec!["api", "v1", "users"],
        vec!["api", "v2"],
    ]
}
fn tree() -> ArrayTrie<usize> {
    let mut t = ArrayTrie::new();
    for (i, p) in paths().iter().enumerate() {
        t.set(p, i)
    }
    t
}
fn sorted(values: Vec<&usize>) -> String {
    let mut v: Vec<_> = values.into_iter().copied().collect();
    v.sort();
    v.into_iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(",")
}
#[test]
fn source_vectors_and_explicit_upstream_bug_fixes() {
    let fixtures: BTreeMap<_, _> = include_str!("swift.tsv")
        .lines()
        .map(|s| s.split_once('\t').unwrap())
        .collect();
    let t = tree();
    assert_eq!(sorted(t.all_values()), fixtures["all"]);
    assert_eq!(sorted(t.child_values()), fixtures["child"]);
    for p in ["", "a", "ab", "api", "ap", "z"] {
        assert_eq!(
            sorted(t.values_along_path(p).into_iter().map(|(_, v)| v).collect()),
            fixtures[format!("along:{p}").as_str()]
        );
        let traversed = t
            .traverse_prefix(p)
            .map(|t| sorted(t.all_values()))
            .unwrap_or("nil".into());
        assert_eq!(traversed, fixtures[format!("traverse:{p}").as_str()]);
        // Swift returns duplicate root/ancestor values. The Rust API intentionally returns each value once.
        let expected: BTreeSet<_> = fixtures[format!("exclude:{p}").as_str()]
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.parse::<usize>().unwrap())
            .collect();
        let actual: BTreeSet<_> = t.values_excluding_prefix(p).into_iter().copied().collect();
        assert_eq!(actual, expected);
    }
    assert_eq!(fixtures["exclude:a"], "0,0,3,4");
    assert_eq!(sorted(t.values_excluding_prefix("a")), "0,3,4");
    let mut t = tree();
    for (i, p) in paths().iter().enumerate() {
        t = t.deleting(p);
        if i < 6 {
            assert_eq!(
                sorted(t.all_values()),
                fixtures[format!("delete:{i}").as_str()]
            );
        }
    }
    // Swift silently misses deletion of a descendant after ancestor compression. Rust removes all paths.
    assert_eq!(fixtures["delete:7"], "6");
    assert!(t.is_empty());
}
#[test]
fn unicode_and_value_subtrie() {
    let mut t = ArrayTrie::new();
    t.set(&["é"], 1);
    t.set(&["e\u{301}", "x"], 2);
    assert_eq!(t.get(&["é", "x"]), Some(&2));
    assert_eq!(t.child_keys().len(), 1);
    let (subtree, v) = t.values_along_path("é").pop().unwrap();
    assert_eq!(*v, 1);
    assert_eq!(subtree.get::<&str>(&[]), None);
    assert_eq!(subtree.get(&["x"]), Some(&2));
    assert_eq!(t.deleting(&["e\u{301}"]).get(&["é", "x"]), Some(&2));
}
