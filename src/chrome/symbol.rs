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
    /// Navigate or point downward.
    ArrowDown,
    /// Navigate or point leftward.
    ArrowLeft,
    /// Navigate or point rightward.
    ArrowRight,
    /// Navigate or point upward.
    ArrowUp,
    /// Collapse an open disclosure.
    Collapse,
    /// Confirm or accept a pending operation.
    Confirm,
    /// Copy the targeted value into a clipboard or application copy buffer.
    Copy,
    /// Decrease a scalar by one application-defined quantum.
    Decrement,
    /// Permanently delete the targeted durable member.
    Delete,
    /// Duplicate an existing member.
    Duplicate,
    /// Expand a closed disclosure.
    Expand,
    /// Export the targeted durable object.
    ///
    /// Its heavy northeast stroke remains legible at every mechanism gauge and
    /// conveys outward motion without borrowing upload's source bar.
    Export,
    /// Open contextual help and keyboard guidance.
    Help,
    /// Mark an item with the canonical heart.
    Heart,
    /// Increase a scalar by one application-defined quantum.
    Increment,
    /// Insert a value from a clipboard or application copy buffer.
    Paste,
    /// Reapply the next command-history entry withdrawn by an undo.
    Redo,
    /// Remove, clear, or dismiss the targeted member.
    ///
    /// Its heavy multiplication cut is centered and D₄-symmetric: every arm
    /// retains equal weight under quarter-turns and reflections.
    Remove,
    /// Rename the targeted member.
    ///
    /// Its sparse diagonal pencil remains legible beneath the armory's physical
    /// soot keyline without the false weight of a faceted barrel.
    Rename,
    /// Restore a withdrawn or resettable member.
    Restore,
    /// Write the current object's pending changes to durable storage.
    ///
    /// Its monochrome hard-shell disk outline avoids platform emoji
    /// substitution while retaining the standard save convention.
    Save,
    /// Open application settings.
    Settings,
    /// Withdraw the most recent command-history entry.
    Undo,
    /// Expose or conceal the targeted object in the current presentation.
    Visibility,
}

impl Symbol {
    /// Complete armory in stable presentation order.
    pub const ALL: [Self; 25] = [
        Self::Add,
        Self::ArrowLeft,
        Self::ArrowRight,
        Self::ArrowUp,
        Self::ArrowDown,
        Self::Remove,
        Self::Delete,
        Self::Duplicate,
        Self::Copy,
        Self::Paste,
        Self::Rename,
        Self::Confirm,
        Self::Save,
        Self::Undo,
        Self::Redo,
        Self::Expand,
        Self::Collapse,
        Self::Export,
        Self::Visibility,
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
            Self::Add | Self::Increment => '✚',
            Self::ArrowDown => '↓',
            Self::ArrowLeft => '←',
            Self::ArrowRight => '→',
            Self::ArrowUp => '↑',
            Self::Collapse => '▾',
            Self::Confirm => '✓',
            Self::Copy => '🗐',
            Self::Decrement => '−',
            Self::Delete => '🗑',
            Self::Duplicate => '⧉',
            Self::Expand => '▸',
            Self::Export => '➚',
            Self::Help => '?',
            Self::Heart => '♥',
            Self::Paste => '📋',
            Self::Remove => '✖',
            Self::Redo => '↷',
            Self::Rename => '🖉',
            Self::Restore => '↺',
            Self::Save => '🖫',
            Self::Settings => '⚙',
            Self::Undo => '↶',
            Self::Visibility => '👁',
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
            | Self::ArrowDown
            | Self::ArrowLeft
            | Self::ArrowRight
            | Self::ArrowUp
            | Self::Collapse
            | Self::Confirm
            | Self::Copy
            | Self::Decrement
            | Self::Duplicate
            | Self::Expand
            | Self::Export
            | Self::Help
            | Self::Increment
            | Self::Paste
            | Self::Redo
            | Self::Remove
            | Self::Rename
            | Self::Restore
            | Self::Save
            | Self::Settings
            | Self::Undo
            | Self::Visibility => MonoglyphFinish::BrightCut,
        }
    }

    /// Stable action name for galleries and instrumentation.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Add => "ADD",
            Self::ArrowDown => "DOWN",
            Self::ArrowLeft => "LEFT",
            Self::ArrowRight => "RIGHT",
            Self::ArrowUp => "UP",
            Self::Collapse => "COLLAPSE",
            Self::Confirm => "CONFIRM",
            Self::Copy => "COPY",
            Self::Decrement => "DECREMENT",
            Self::Delete => "DELETE",
            Self::Duplicate => "DUPLICATE",
            Self::Expand => "EXPAND",
            Self::Export => "EXPORT",
            Self::Help => "HELP",
            Self::Heart => "HEART",
            Self::Increment => "INCREMENT",
            Self::Paste => "PASTE",
            Self::Redo => "REDO",
            Self::Remove => "REMOVE",
            Self::Rename => "RENAME",
            Self::Restore => "RESTORE",
            Self::Save => "SAVE",
            Self::Settings => "SETTINGS",
            Self::Undo => "UNDO",
            Self::Visibility => "VISIBILITY",
        }
    }
}
