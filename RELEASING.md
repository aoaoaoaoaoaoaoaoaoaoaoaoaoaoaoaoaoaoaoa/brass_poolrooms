# Releasing

A Poolrooms version identifies one reconstructible source commit. Releases are
cut only from `main` after the complete native and WebGPU gate passes.

1. Set the shared `brass_foundry`/`brass_poolrooms` workspace version and README
   dependency examples to the new version.
2. Run `./check.py verify`.
3. Commit every intended source, documentation, gallery, and lockfile change,
   then package both crates from that clean commit.
4. Push `main`, create the annotated `v<version>` tag on that exact commit, and
   push the tag.
5. Run `scripts/release <version>` to repeat the crate gate and assemble the
   exact-version Web Kit without publication.
6. Run `scripts/release <version> publish` to publish `brass_foundry`, wait for
   that registry boundary, publish `brass_poolrooms`, and publish the signed Web
   Kit GitHub Release assets from the already tagged source.
7. Verify the registry version and release assets, then advance Eternalist Apps
   and application lockfiles in dependency order.

The release command rejects a dirty checkout, a detached or non-`main` branch,
an unpushed commit, a missing or misplaced tag, and a manifest-version
mismatch. Its publication steps are restartable after a partial registry
crossing. `cargo publish --allow-dirty` is forbidden.
