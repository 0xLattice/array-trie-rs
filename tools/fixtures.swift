import Foundation
import ArrayTrie
var trie=ArrayTrie<Int>()
let paths:[[String]]=[[],["a"],["ab"],["b"],["c"],["api","v1"],["api","v1","users"],["api","v2"]]
for(i,path)in paths.enumerated(){trie.set(path,value:i)}
func emit(_ label:String,_ values:[Int]){print(label+"\t"+values.sorted().map(String.init).joined(separator:","))}
emit("all",trie.allValues());emit("child",trie.childValues())
for path in ["","a","ab","api","ap","z"] {
 emit("exclude:"+path,trie.valuesExcludingPrefix(path));emit("along:"+path,trie.valuesAlongPath(path).map{$0.1})
 if let t=trie.traverse(path:path){emit("traverse:"+path,t.allValues())}else{print("traverse:"+path+"\tnil")}
}
for(i,path)in paths.enumerated(){trie=trie.deleting(path:path);emit("delete:\(i)",trie.allValues())}
