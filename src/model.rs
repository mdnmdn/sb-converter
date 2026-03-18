use serde_json::Value;
use std::collections::HashMap;

/// The kind of a cell in a note.
#[derive(Debug, Clone, PartialEq)]
pub enum CellKind {
    Markdown,
    Code { language: String },
    Text,
    Latex,
    Diagram,
    Other(String),
}

/// A single content cell.
#[derive(Debug, Clone)]
pub struct NoteCell {
    pub kind: CellKind,
    pub data: String,
}

/// Provider-agnostic note.
#[derive(Debug, Clone)]
pub struct Note {
    pub title: String,
    pub tags: Vec<String>,
    /// Unix timestamp of last update (0 if unknown).
    pub updated_at: u64,
    /// Logical path inside the source (e.g. "Folder/Notebook").
    pub path: String,
    /// Source identifier (library name, repo URL, …).
    pub source: String,
    pub cells: Vec<NoteCell>,
    /// Provider-specific extra data (e.g. uuid, quiver_file, …).
    pub custom_data: HashMap<String, Value>,
}
