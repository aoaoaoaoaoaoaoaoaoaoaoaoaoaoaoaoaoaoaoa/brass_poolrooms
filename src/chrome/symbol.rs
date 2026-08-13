//! Canonical action symbology for foundry-owned one-glyph controls.

#![deny(missing_docs)]

/// A common action mark admitted by the Poolrooms symbology armory.
///
/// The enum owns the semantic-action-to-glyph decision. Passing it through
/// [`super::Monoglyph::symbol`] also subjects every mark to the same forged
/// small, medium, or large typographic die. Applications retain
/// [`super::Monoglyph::new`] for genuinely product-specific marks.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Symbol {
    /// Add a new member to a collection.
    Add,
    /// Collapse an open disclosure.
    Collapse,
    /// Confirm or accept a pending operation.
    Confirm,
    /// Decrease a scalar by one application-defined quantum.
    Decrement,
    /// Duplicate an existing member.
    Duplicate,
    /// Expand a closed disclosure.
    Expand,
    /// Open contextual help and keyboard guidance.
    Help,
    /// Increase a scalar by one application-defined quantum.
    Increment,
    /// Remove, clear, or dismiss the targeted member.
    Remove,
    /// Rename the targeted member.
    Rename,
    /// Restore a withdrawn or resettable member.
    Restore,
}

impl Symbol {
    /// Complete armory in stable presentation order.
    pub const ALL: [Self; 11] = [
        Self::Add,
        Self::Remove,
        Self::Duplicate,
        Self::Rename,
        Self::Confirm,
        Self::Expand,
        Self::Collapse,
        Self::Help,
        Self::Restore,
        Self::Increment,
        Self::Decrement,
    ];

    /// Canonical Unicode scalar cut for this action.
    pub const fn glyph(self) -> char {
        match self {
            Self::Add | Self::Increment => '+',
            Self::Collapse => '▾',
            Self::Confirm => '✓',
            Self::Decrement => '−',
            Self::Duplicate => '⧉',
            Self::Expand => '▸',
            Self::Help => '?',
            Self::Remove => '×',
            Self::Rename => '✎',
            Self::Restore => '↺',
        }
    }

    /// Stable action name for galleries and instrumentation.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Add => "ADD",
            Self::Collapse => "COLLAPSE",
            Self::Confirm => "CONFIRM",
            Self::Decrement => "DECREMENT",
            Self::Duplicate => "DUPLICATE",
            Self::Expand => "EXPAND",
            Self::Help => "HELP",
            Self::Increment => "INCREMENT",
            Self::Remove => "REMOVE",
            Self::Rename => "RENAME",
            Self::Restore => "RESTORE",
        }
    }
}
