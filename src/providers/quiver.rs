use std::{collections::HashMap, fs, path::{Path, PathBuf}};
use serde::Deserialize;
use serde_json::json;

use crate::{
    model::{CellKind, Note, NoteCell},
    provider::Provider,
};

#[derive(Deserialize)]
struct LibNode {
    uuid: String,
    #[serde(default)]
    children: Vec<LibNode>,
}
#[derive(Deserialize)]
struct LibMeta {
    children: Vec<LibNode>,
}
#[derive(Deserialize)]
struct NotebookMeta {
    name: String,
}
#[derive(Deserialize)]
struct NoteMeta {
    title: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    updated_at: u64,
    uuid: String,
}
#[derive(Deserialize)]
struct RawCell {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    language: String,
    data: String,
}
#[derive(Deserialize)]
struct RawContent {
    cells: Vec<RawCell>,
}

pub struct QuiverProvider {
    pub lib_path: PathBuf,
}

impl QuiverProvider {
    pub fn new(lib_path: impl Into<PathBuf>) -> Self {
        Self { lib_path: lib_path.into() }
    }

    fn notebook_names(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for entry in fs::read_dir(&self.lib_path).unwrap().flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("qvnotebook") {
                let stem = p.file_stem().unwrap().to_string_lossy().to_string();
                if let Ok(s) = fs::read_to_string(p.join("meta.json")) {
                    if let Ok(m) = serde_json::from_str::<NotebookMeta>(&s) {
                        map.insert(stem, m.name);
                        continue;
                    }
                }
                map.entry(stem.clone()).or_insert(stem);
            }
        }
        map
    }

    fn walk(
        &self,
        nodes: &[LibNode],
        names: &HashMap<String, String>,
        prefix: &str,
        out: &mut Vec<Note>,
    ) {
        let lib_name = self.lib_path.file_name().unwrap().to_string_lossy().to_string();
        for node in nodes {
            let nb_dir = self.lib_path.join(format!("{}.qvnotebook", node.uuid));
            if !nb_dir.exists() { continue; }
            let name = names.get(&node.uuid).cloned().unwrap_or(node.uuid.clone());
            if name == "Trash" { continue; }
            let path = if prefix.is_empty() { name.clone() } else { format!("{prefix}/{name}") };

            for entry in fs::read_dir(&nb_dir).unwrap().flatten() {
                let p = entry.path();
                if p.extension().and_then(|e| e.to_str()) == Some("qvnote") {
                    if let Some(note) = self.read_note(&p, &path, &lib_name) {
                        out.push(note);
                    }
                }
            }
            if !node.children.is_empty() {
                self.walk(&node.children, names, &path, out);
            }
        }
    }

    fn read_note(&self, note_dir: &Path, path: &str, lib_name: &str) -> Option<Note> {
        let meta: NoteMeta = serde_json::from_str(
            &fs::read_to_string(note_dir.join("meta.json")).ok()?
        ).ok()?;
        let content: RawContent = serde_json::from_str(
            &fs::read_to_string(note_dir.join("content.json")).ok()?
        ).ok()?;

        let cells = content.cells.into_iter().map(|c| NoteCell {
            kind: match c.kind.as_str() {
                "code"    => CellKind::Code { language: c.language },
                "text"    => CellKind::Text,
                "latex"   => CellKind::Latex,
                "diagram" => CellKind::Diagram,
                "markdown"=> CellKind::Markdown,
                other     => CellKind::Other(other.to_string()),
            },
            data: c.data,
        }).collect();

        let mut custom_data = HashMap::new();
        custom_data.insert("uuid".into(), json!(meta.uuid));
        custom_data.insert("quiver_file".into(), json!(lib_name));

        Some(Note {
            title: meta.title,
            tags: meta.tags,
            updated_at: meta.updated_at,
            path: path.to_string(),
            source: lib_name.to_string(),
            cells,
            custom_data,
        })
    }
}

impl Provider for QuiverProvider {
    fn name(&self) -> &str { "quiver" }
    fn readonly(&self) -> bool { true }

    fn read_notes(&self) -> Vec<Note> {
        let lib_meta: LibMeta = serde_json::from_str(
            &fs::read_to_string(self.lib_path.join("meta.json")).unwrap()
        ).unwrap();
        let names = self.notebook_names();
        let mut notes = Vec::new();
        self.walk(&lib_meta.children, &names, "", &mut notes);
        notes
    }
}
