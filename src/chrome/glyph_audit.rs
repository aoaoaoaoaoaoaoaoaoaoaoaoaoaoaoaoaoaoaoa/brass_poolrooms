//! Development gate for authored Rust literals against the shipped face's cmap.
//! User data and documentation are not an authored-UI coverage contract.

use std::{collections::BTreeSet, error::Error, fs, path::Path};

use proc_macro2::{Span, TokenStream, TokenTree};
use syn::{
    Attribute, ItemFn, ItemMod, Lit, Macro,
    visit::{self, Visit},
};
use ttf_parser::Face;

/// Audit string and character literals, including escaped scalars and macro
/// arguments, beneath a Rust source file or directory. Test bodies and
/// attributes are excluded. Unsupported scalars name their source and line;
/// add an original glyph to the owned map or deliberately change the symbol.
///
/// Enable `glyph-audit` only in development dependencies. Call this from a
/// consuming application's normal gate to cover its authored vocabulary.
pub fn check_tree(root: &Path) -> Result<(), Box<dyn Error>> {
    assert!(
        egui::FontDefinitions::default().font_data.is_empty(),
        "egui's bundled fallback fonts are enabled; disable default features on egui dependencies"
    );
    let face = Face::parse(super::TYPEFACE, 0)?;
    let mut missing = BTreeSet::new();
    scan(root, &face, &mut missing)?;
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Unmapped UI glyphs:\n{}",
            missing.into_iter().collect::<Vec<_>>().join("\n")
        )
        .into())
    }
}

fn scan(
    path: &Path,
    face: &Face<'_>,
    missing: &mut BTreeSet<String>,
) -> Result<(), Box<dyn Error>> {
    if fs::metadata(path)?.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if !entry.file_type()?.is_symlink() {
                scan(&entry.path(), face, missing)?;
            }
        }
    } else if path.extension().is_some_and(|extension| extension == "rs") {
        let source = syn::parse_file(&fs::read_to_string(path)?)?;
        Audit {
            face,
            path,
            missing,
        }
        .visit_file(&source);
    }
    Ok(())
}

struct Audit<'a> {
    face: &'a Face<'a>,
    path: &'a Path,
    missing: &'a mut BTreeSet<String>,
}

impl Audit<'_> {
    fn text(&mut self, value: &str, span: Span) {
        for scalar in value.chars().filter(|scalar| !scalar.is_control()) {
            if self.face.glyph_index(scalar).is_none() {
                let _ = self.missing.insert(format!(
                    "{}:{}: U+{:04X} {scalar}",
                    self.path.display(),
                    span.start().line,
                    u32::from(scalar)
                ));
            }
        }
    }

    fn tokens(&mut self, tokens: TokenStream) {
        for token in tokens {
            match token {
                TokenTree::Group(group) => self.tokens(group.stream()),
                TokenTree::Literal(literal) => {
                    if let Ok(literal) = syn::parse2::<Lit>(TokenTree::Literal(literal).into()) {
                        self.visit_lit(&literal);
                    }
                }
                TokenTree::Punct(_) | TokenTree::Ident(_) => {}
            }
        }
    }
}

fn test_only(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.path().is_ident("test")
            || (attribute.path().is_ident("cfg")
                && attribute
                    .meta
                    .require_list()
                    .is_ok_and(|list| list.tokens.to_string() == "test"))
    })
}

impl<'ast> Visit<'ast> for Audit<'_> {
    fn visit_attribute(&mut self, _: &'ast Attribute) {}

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        if !test_only(&function.attrs) {
            visit::visit_item_fn(self, function);
        }
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        if !test_only(&module.attrs) {
            visit::visit_item_mod(self, module);
        }
    }

    fn visit_lit(&mut self, literal: &'ast Lit) {
        match literal {
            Lit::Str(value) => self.text(&value.value(), value.span()),
            Lit::Char(value) => self.text(&value.value().to_string(), value.span()),
            _ => {}
        }
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        self.tokens(invocation.tokens.clone());
    }
}

#[test]
fn authored_vocabulary_has_no_fallback() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for tree in ["src", "examples"] {
        check_tree(&root.join(tree))?;
    }
    // Build tools belong to the repository, not the distributable crate.
    let tools = root.join("tools");
    if tools.try_exists()? {
        check_tree(&tools)?;
    }
    Ok(())
}
