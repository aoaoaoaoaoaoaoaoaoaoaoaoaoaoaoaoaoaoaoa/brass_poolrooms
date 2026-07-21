use std::{env, error::Error, io, path::PathBuf};

#[path = "build/checkbox_atlas.rs"]
mod checkbox_atlas;
#[path = "src/chrome/foundry/law.rs"]
mod foundry_law;

// Cargo package verification can overwrite this workspace's build-script unit
// when both use one target directory. Retaining the compilation root turns
// that cache collision into a loud, actionable failure instead of replaying a
// stale atlas.
const COMPILED_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    if manifest_dir != COMPILED_MANIFEST_DIR {
        return Err(io::Error::other(format!(
            "build-script cache crossed source roots: compiled for \
             {COMPILED_MANIFEST_DIR}, invoked for {manifest_dir}; run \
             `cargo clean -p dwemer_poolrooms`, and give `cargo package` a \
             separate --target-dir"
        ))
        .into());
    }
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=build/checkbox_atlas.rs");
    println!("cargo::rerun-if-changed=src/chrome/foundry/law.rs");
    let output = PathBuf::from(env::var("OUT_DIR")?).join("checkbox_atlas.rs");
    checkbox_atlas::bake(&output)?;
    Ok(())
}
