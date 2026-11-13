# Object-tag catalogue differential

`export_reference.py` executes unchanged libfilmshell catalogue lookups from
commit `f050371a6cda83b9887eeb83a1884336fce28e53`, after checking both original
source blobs. A test-only adapter serializes every tag, item/name, direct parent,
vehicle-part association, power-weapon flag and weapon-variant hash. It also
exercises valid/invalid grenade type indices. No local catalogue supplies the
expected output. `reference.json` has 2,002 tags and 40 variant hashes.

Reproduce:

```sh
python3 export_reference.py /path/to/libfilmshell > /tmp/items-reference.json
cmp reference.json /tmp/items-reference.json
python3 ../../film/catalog/items/generate.py /path/to/libfilmshell
```

The generator copies the pinned data blob with import/derive/format adaptations.
The source says the data came from game files, but supplies no game build. Local
queries use the single bundled `Catalog`; its data does not establish an
applicable game build. This catalogue does not admit wire layouts, select vehicle
physics branches or prove instance ownership. Asset tags, variant string hashes
and the existing 64-bit weapon IDs remain separate types/namespaces. Unknown and
unnamed entries and direct relation targets remain unchanged.
