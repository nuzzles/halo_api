# Damage and sandbox reference catalogue

The pinned source is libfilmshell `f050371a6cda83b9887eeb83a1884336fce28e53`.
`export_reference.py` checks four original Git blobs, then runs unchanged lookup
functions and damage helpers through test-only serialization adapters. Separate
output files prevent test-runner stdout from interleaving with reference rows.
The export contains all 468 damage effects, 151 firing rows, 294 projectile rows,
64 vehicle bounds, reference Spartan vitality and damage flag constants.

Every floating-point scalar is compared as its exact `f64::to_bits()` value.
This preserves the upstream decimal constants, including their precision limits;
it does not recover original game-file float words. Damage bounds, multipliers,
radius, all 32 individual flags, fixed-amount results, absent amounts, optional
weapon/impact/projectile links and ordered projectile lists remain distinct.

```sh
python3 export_reference.py /path/to/libfilmshell > /tmp/reference.json
cmp reference.json /tmp/reference.json
python3 generate.py /path/to/libfilmshell
```

The generator copies both data blobs with only imports, attribution and formatting
adaptations. An optional output directory supports reproduction without touching
the checkout. Local APIs require explicit catalogue revision selection and expose
each data blob's provenance. The source does not identify a compatible game build.
These catalogues neither admit wire layouts nor fill gaps in recorded gameplay.
