use crate::model::Note;

/// A provider can read notes from a source and optionally write them back.
pub trait Provider {
    /// Human-readable name (e.g. "quiver", "obsidian-markdown").
    fn name(&self) -> &str;

    /// Whether this provider supports writing notes back.
    fn readonly(&self) -> bool {
        false
    }

    /// Read all notes from the source.
    fn read_notes(&self) -> Vec<Note>;

    /// Write a note to the destination.
    /// Implementations on readonly providers should panic or be left unimplemented.
    fn write_note(&self, note: &Note) {
        let _ = note;
        panic!("provider '{}' is read-only", self.name());
    }
}
