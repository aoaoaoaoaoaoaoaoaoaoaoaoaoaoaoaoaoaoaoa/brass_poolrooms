//! Poolrooms foundry projection for the Web Kit's platform die atlas.

use std::{env, error::Error, path::PathBuf};

#[allow(dead_code)]
mod foundry_law {
    include!(concat!(
        env!("POOLROOMS_SOURCE"),
        "/src/chrome/foundry/law.rs"
    ));
}

#[allow(dead_code, unused_imports)]
mod foundry {
    include!(concat!(env!("POOLROOMS_SOURCE"), "/build/foundry_atlas.rs"));
    include!("dies.rs");
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1).map(PathBuf::from);
    let masks = args.next().ok_or("missing relief-mask directory")?;
    let output = args.next().ok_or("missing output PAM path")?;
    if args.next().is_some() {
        return Err("usage: brass-poolrooms-web-platform-die-forge MASK_DIR OUTPUT.pam".into());
    }
    foundry::forge(&masks, &output)?;
    Ok(())
}
