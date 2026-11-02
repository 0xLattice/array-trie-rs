#!/usr/bin/env python3
"""Compile the pinned sibling Swift source and record differential observations."""
from pathlib import Path
import subprocess, tempfile
repo = Path(__file__).resolve().parents[1]
base = repo.parent
with tempfile.TemporaryDirectory(prefix="lattice-swift-fixtures-") as directory:
    out = Path(directory)
    sources = sorted(str(p) for p in (base / "TrieDictionary/Sources/TrieDictionary").glob("*.swift"))
    subprocess.run(["swiftc", "-emit-library", "-emit-module", "-module-name", "TrieDictionary", *sources, "-emit-module-path", str(out / "TrieDictionary.swiftmodule"), "-o", str(out / "libTrieDictionary.dylib")], check=True)
    libraries = ["-lTrieDictionary"]
    subprocess.run(["swiftc", "-emit-library", "-emit-module", "-module-name", "ArrayTrie", "-I", str(out), "-L", str(out), "-lTrieDictionary", str(base / "ArrayTrie/Sources/ArrayTrie/ArrayTrie.swift"), "-emit-module-path", str(out / "ArrayTrie.swiftmodule"), "-o", str(out / "libArrayTrie.dylib")], check=True)
    libraries.append("-lArrayTrie")
    subprocess.run(["swiftc", "-I", str(out), "-L", str(out), *libraries, "-Xlinker", "-rpath", "-Xlinker", str(out), str(repo / "tools/fixtures.swift"), "-o", str(out / "fixtures")], check=True)
    with (repo / "tests/swift.tsv").open("w") as output:
        subprocess.run([str(out / "fixtures")], stdout=output, check=True)
