# Releasing

A Poolrooms version identifies one reconstructible source commit. Releases are
cut only from `main` after the complete native and WebGPU gate passes.

1. Set the workspace version and README dependency examples to the new version.
2. Run `./check.py verify`.
3. Commit every intended source, documentation, gallery, and lockfile change,
   then run `cargo package --locked` from that clean commit.
4. Push `main`, create the annotated `v<version>` tag on that exact commit, and
   push the tag.
5. Run `scripts/release <version>` to repeat the package gate without
   publication.
6. Run `scripts/release <version> publish` to publish the already tagged source.
7. Verify the registry version, then advance Eternalist Apps and application
   lockfiles in dependency order.

The release command rejects a dirty checkout, a detached or non-`main` branch,
an unpushed commit, a missing or misplaced tag, and a manifest-version mismatch.
`cargo publish --allow-dirty` is forbidden.
