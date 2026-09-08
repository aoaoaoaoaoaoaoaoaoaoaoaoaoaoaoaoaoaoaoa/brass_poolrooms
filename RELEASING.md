# Releasing

A Poolrooms version identifies one reconstructible source commit. Releases are
cut only from `main` after the complete native and WebGPU gate passes.

Releases are cut by the Poolrooms line's release engine from the line root:

```sh
scripts/release brass_poolrooms <version>            # gate only, tree restored
scripts/release brass_poolrooms <version> --publish  # bump, gate, tag, publish, push
```

The engine sets the shared `brass_foundry`/`brass_poolrooms` version and the
README dependency examples, locks the Web Chrome forge to it, ratifies only the
lockfile line of `web-kit/projection.sources` (a stale source digest must be
inspected and ratified by hand first), runs every Foundry proof this host can
prove, assembles the exact-version Web Kit, signs the tag, publishes
`brass_foundry`, waits for that registry boundary, publishes `brass_poolrooms`,
pushes `main` and the tag, publishes the signed Web Kit GitHub Release assets,
and holds one Vigil wait on the tag's hosted run.

The engine rejects a dirty checkout, a non-`main` branch, an unpushed
`origin/main`, an existing tag, a `[patch]` section, and a version that does
not advance. Its publication steps are restartable after a partial registry
crossing. `cargo publish --allow-dirty` is forbidden.
