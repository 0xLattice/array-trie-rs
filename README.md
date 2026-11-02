# lattice-array-trie

Path-segment trie port of 0xLattice/ArrayTrie. Upstream SHA is in UPSTREAM and original license is retained.

Implemented: root values, segment set/get/delete, segment and character-prefix traversal, child keys/characters/prefix, value collection and exclusion, merging and merge-all. Tests cover persistent deletion, root values, prefix segmentation and merging.

Remaining parity: compressed path storage, all traversal edge cases and the complete upstream test corpus. Unicode canonical-equivalent matching is implemented. `tests/swift_compat.rs` compares actual Swift observations, explicitly recording upstream duplicate-value and failed-descendant-deletion bugs and testing the corrected Rust behavior. Current storage is an explicit segment trie; this preserves tested lookup behavior but has not established performance or full API parity.


## Deliberate fixes to upstream defects

With root 0, a=1, ab=2, b=3, c=4, and nested api paths, Swift prefix exclusion returns duplicates (for prefix a: `[0,0,3,4]`). Rust returns each semantic value once (`[0,3,4]`). Swift deleting an ancestor value and then its formerly compressed descendant can silently retain that descendant; Rust removes the requested exact path. Composed/decomposed Unicode first segments can cause Swift prefix exclusion to trap because TrieDictionary hashes the raw first scalar while Character equality is canonical; Rust compares canonical graphemes and does not trap. These are explicit compatibility differences, not passing byte-for-byte parity claims.
