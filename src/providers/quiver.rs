use serde::Deserialize;
use serde_json::json;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

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
        Self {
            lib_path: lib_path.into(),
        }
    }

    fn notebook_names(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for entry in fs::read_dir(&self.lib_path).unwrap().flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("qvnotebook") {
                let stem = p.file_stem().unwrap().to_string_lossy().to_string();
                if let Ok(s) = fs::read_to_string(p.join("meta.json"))
                    && let Ok(m) = serde_json::from_str::<NotebookMeta>(&s)
                {
                    map.insert(stem, m.name);
                    continue;
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
        let lib_name = self
            .lib_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        for node in nodes {
            let nb_dir = self.lib_path.join(format!("{}.qvnotebook", node.uuid));
            if !nb_dir.exists() {
                continue;
            }
            let name = names.get(&node.uuid).cloned().unwrap_or(node.uuid.clone());
            if name == "Trash" {
                continue;
            }
            let path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}/{name}")
            };

            for entry in fs::read_dir(&nb_dir).unwrap().flatten() {
                let p = entry.path();
                if p.extension().and_then(|e| e.to_str()) == Some("qvnote")
                    && let Some(note) = self.read_note(&p, &path, &lib_name)
                {
                    out.push(note);
                }
            }
            if !node.children.is_empty() {
                self.walk(&node.children, names, &path, out);
            }
        }
    }

    fn read_note(&self, note_dir: &Path, path: &str, lib_name: &str) -> Option<Note> {
        let meta: NoteMeta =
            serde_json::from_str(&fs::read_to_string(note_dir.join("meta.json")).ok()?).ok()?;
        let content: RawContent =
            serde_json::from_str(&fs::read_to_string(note_dir.join("content.json")).ok()?).ok()?;

        let cells = content
            .cells
            .into_iter()
            .map(|c| NoteCell {
                kind: match c.kind.as_str() {
                    "code" => CellKind::Code {
                        language: c.language,
                    },
                    "text" => CellKind::Text,
                    "latex" => CellKind::Latex,
                    "diagram" => CellKind::Diagram,
                    "markdown" => CellKind::Markdown,
                    other => CellKind::Other(other.to_string()),
                },
                data: c.data,
            })
            .collect();

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

impl QuiverProvider {
    /// Resolve or create a notebook directory for the given logical name.
    /// Reuses an existing notebook with the same name, or creates a new one with a fresh UUID.
    fn notebook_dir_for(&self, name: &str) -> PathBuf {
        // check existing notebooks
        if let Ok(entries) = fs::read_dir(&self.lib_path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|e| e.to_str()) == Some("qvnotebook")
                    && let Ok(s) = fs::read_to_string(p.join("meta.json"))
                    && let Ok(m) = serde_json::from_str::<NotebookMeta>(&s)
                    && m.name == name
                {
                    return p;
                }
            }
        }
        // create new notebook
        let uuid = new_uuid();
        let nb_dir = self.lib_path.join(format!("{uuid}.qvnotebook"));
        fs::create_dir_all(&nb_dir).unwrap();
        fs::write(
            nb_dir.join("meta.json"),
            serde_json::to_string(&json!({ "name": name, "uuid": uuid })).unwrap(),
        )
        .unwrap();
        self.update_lib_meta(&uuid);
        nb_dir
    }

    /// Add a notebook UUID to the library meta.json if not already present.
    fn update_lib_meta(&self, uuid: &str) {
        let meta_path = self.lib_path.join("meta.json");
        let mut meta: serde_json::Value = fs::read_to_string(&meta_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| json!({ "uuid": "library", "children": [] }));

        let children = meta["children"].as_array_mut().unwrap();
        if !children.iter().any(|c| c["uuid"] == uuid) {
            children.push(json!({ "uuid": uuid }));
        }
        fs::write(&meta_path, serde_json::to_string_pretty(&meta).unwrap()).unwrap();
    }
}

fn new_uuid() -> String {
    // simple UUID v4-like from random bytes via std
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;
    let mut h = DefaultHasher::new();
    SystemTime::now().hash(&mut h);
    std::thread::current().id().hash(&mut h);
    format!(
        "{:016X}-{:04X}-{:04X}-{:04X}-{:012X}",
        h.finish(),
        h.finish() >> 16 & 0xffff,
        0x4000 | (h.finish() >> 12 & 0x0fff),
        0x8000 | (h.finish() >> 14 & 0x3fff),
        h.finish() & 0xffffffffffff
    )
}

impl Provider for QuiverProvider {
    fn name(&self) -> &str {
        "quiver"
    }
    fn readonly(&self) -> bool {
        false
    }

    fn read_notes(&self) -> Vec<Note> {
        let lib_meta: LibMeta =
            serde_json::from_str(&fs::read_to_string(self.lib_path.join("meta.json")).unwrap())
                .unwrap();
        let names = self.notebook_names();
        let mut notes = Vec::new();
        self.walk(&lib_meta.children, &names, "", &mut notes);
        notes
    }

    fn write_note(&self, note: &Note) {
        fs::create_dir_all(&self.lib_path).unwrap();

        // ensure library meta.json exists
        let lib_meta_path = self.lib_path.join("meta.json");
        if !lib_meta_path.exists() {
            let lib_name = self.lib_path.file_stem().unwrap().to_string_lossy();
            fs::write(
                &lib_meta_path,
                serde_json::to_string(&json!({ "uuid": lib_name, "children": [] })).unwrap(),
            )
            .unwrap();
        }

        // top-level notebook name (first path segment)
        let notebook_name = note.path.split('/').next().unwrap_or(&note.path);
        let nb_dir = self.notebook_dir_for(notebook_name);

        // reuse existing note UUID if present in custom_data
        let uuid = note
            .custom_data
            .get("uuid")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(new_uuid);

        let note_dir = nb_dir.join(format!("{uuid}.qvnote"));
        fs::create_dir_all(&note_dir).unwrap();

        // meta.json — skip rewrite if updated_at unchanged
        let meta_path = note_dir.join("meta.json");
        let existing_ts: Option<u64> = fs::read_to_string(&meta_path)
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .and_then(|v| v["updated_at"].as_u64());
        if existing_ts == Some(note.updated_at) {
            return;
        }

        let meta = json!({
            "title": note.title,
            "tags": note.tags,
            "updated_at": note.updated_at,
            "created_at": note.updated_at,
            "uuid": uuid,
        });
        fs::write(&meta_path, serde_json::to_string_pretty(&meta).unwrap()).unwrap();

        // content.json
        let cells: Vec<serde_json::Value> = note
            .cells
            .iter()
            .map(|c| match &c.kind {
                CellKind::Code { language } => {
                    json!({ "type": "code", "language": language, "data": c.data })
                }
                CellKind::Text => json!({ "type": "text",    "data": c.data }),
                CellKind::Latex => json!({ "type": "latex",   "data": c.data }),
                CellKind::Diagram => json!({ "type": "diagram", "data": c.data }),
                CellKind::Markdown => json!({ "type": "markdown", "data": c.data }),
                CellKind::Other(t) => json!({ "type": t, "data": c.data }),
            })
            .collect();

        let content = json!({ "title": note.title, "cells": cells });
        fs::write(
            note_dir.join("content.json"),
            serde_json::to_string_pretty(&content).unwrap(),
        )
        .unwrap();
    }
}
