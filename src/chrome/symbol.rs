//! Canonical action symbology for foundry-owned one-glyph controls.

#![deny(missing_docs)]

use super::MonoglyphFinish;

/// A common action mark admitted by the Poolrooms symbology armory.
///
/// The enum owns the semantic-action-to-glyph decision. Passing it through
/// [`super::Monoglyph::symbol`] also subjects every mark to the same forged
/// small, medium, or large typographic die. Applications retain
/// [`super::Monoglyph::new`] for genuinely product-specific marks.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Symbol {
    /// Add a new member to a collection.
    ///
    /// Its heavy plus cut is centered and D₄-symmetric, and is shared with
    /// [`Self::Increment`].
    Add,
    /// Collapse an open disclosure.
    Collapse,
    /// Confirm or accept a pending operation.
    Confirm,
    /// Decrease a scalar by one application-defined quantum.
    Decrement,
    /// Permanently delete the targeted durable member.
    Delete,
    /// Duplicate an existing member.
    Duplicate,
    /// Expand a closed disclosure.
    Expand,
    /// Open contextual help and keyboard guidance.
    Help,
    /// Mark an item with the canonical heart.
    Heart,
    /// Increase a scalar by one application-defined quantum.
    Increment,
    /// Remove, clear, or dismiss the targeted member.
    ///
    /// Its heavy multiplication cut is centered and D₄-symmetric: every arm
    /// retains equal weight under quarter-turns and reflections.
    Remove,
    /// Rename the targeted member.
    Rename,
    /// Restore a withdrawn or resettable member.
    Restore,
    /// Open application settings.
    Settings,
}

impl Symbol {
    /// Complete armory in stable presentation order.
    pub const ALL: [Self; 14] = [
        Self::Add,
        Self::Remove,
        Self::Delete,
        Self::Duplicate,
        Self::Rename,
        Self::Confirm,
        Self::Expand,
        Self::Collapse,
        Self::Help,
        Self::Heart,
        Self::Restore,
        Self::Settings,
        Self::Increment,
        Self::Decrement,
    ];

    /// Canonical Unicode scalar cut for this action.
    pub const fn glyph(self) -> char {
        match self {
            Self::Add | Self::Increment => '➕',
            Self::Collapse => '▾',
            Self::Confirm => '✓',
            Self::Decrement => '−',
            Self::Delete => '🗑',
            Self::Duplicate => '⧉',
            Self::Expand => '▸',
            Self::Help => '?',
            Self::Heart => '♥',
            Self::Remove => '✖',
            Self::Rename => '✎',
            Self::Restore => '↺',
            Self::Settings => '⚙',
        }
    }

    /// Default physical finish selected by this action's semantics.
    ///
    /// Consumers may override the result through
    /// [`super::Monoglyph::finish`]. Raw Unicode monoglyphs do not consult
    /// this table and remain bright-cut by default.
    pub const fn default_finish(self) -> MonoglyphFinish {
        match self {
            Self::Delete => MonoglyphFinish::Danger,
            Self::Heart => MonoglyphFinish::Love,
            Self::Add
            | Self::Collapse
            | Self::Confirm
            | Self::Decrement
            | Self::Duplicate
            | Self::Expand
            | Self::Help
            | Self::Increment
            | Self::Remove
            | Self::Rename
            | Self::Restore
            | Self::Settings => MonoglyphFinish::BrightCut,
        }
    }

    /// Stable action name for galleries and instrumentation.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Add => "ADD",
            Self::Collapse => "COLLAPSE",
            Self::Confirm => "CONFIRM",
            Self::Decrement => "DECREMENT",
            Self::Delete => "DELETE",
            Self::Duplicate => "DUPLICATE",
            Self::Expand => "EXPAND",
            Self::Help => "HELP",
            Self::Heart => "HEART",
            Self::Increment => "INCREMENT",
            Self::Remove => "REMOVE",
            Self::Rename => "RENAME",
            Self::Restore => "RESTORE",
            Self::Settings => "SETTINGS",
        }
    }
}
