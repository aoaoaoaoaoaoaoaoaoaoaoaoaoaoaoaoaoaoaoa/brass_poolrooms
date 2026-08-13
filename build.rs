use std::{env, error::Error, io, path::PathBuf};

#[path = "build/foundry_atlas.rs"]
mod foundry_atlas;
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
             `cargo clean -p brass_poolrooms`, and give `cargo package` a \
             separate --target-dir"
        ))
        .into());
    }
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=build/foundry_atlas.rs");
    println!("cargo::rerun-if-changed=src/chrome/foundry/law.rs");
    let output = PathBuf::from(env::var("OUT_DIR")?);
    foundry_atlas::bake(
        &output.join("checkbox_atlas.rs"),
        &output.join("monoglyph_atlas.rs"),
        &output.join("corner_close_atlas.rs"),
        &output.join("drag_handle_atlas.rs"),
        &output.join("number_input_atlas.rs"),
        &output.join("material_study_atlas.rs"),
    )?;
    Ok(())
}
