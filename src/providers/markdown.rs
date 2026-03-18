use std::{collections::HashMap, fs, path::{Path, PathBuf}};
use similar::{ChangeTag, TextDiff};

use crate::{
    model::{CellKind, Note, NoteCell},
    provider::Provider,
};

pub struct MarkdownProvider {
    pub out_dir: PathBuf,
}

impl MarkdownProvider {
    pub fn new(out_dir: impl Into<PathBuf>) -> Self {
        Self { out_dir: out_dir.into() }
    }
}

fn sanitize(s: &str) -> String {
    s.chars().map(|c| if r#"/\:*?"<>|"#.contains(c) { '_' } else { c }).collect()
}

fn existing_updated_at(path: &std::path::Path) -> Option<u64> {
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("quiver_updated: ") {
            return rest.trim().parse().ok();
        }
    }
    None
}

pub fn render_note(note: &Note) -> String {
    let mut md = String::from("---\n");
    if !note.tags.is_empty() {
        md.push_str("tags:");
        for t in &note.tags { md.push_str(&format!("\n  - {t}")); }
        md.push('\n');
    }
    md.push_str(&format!("quiver_file: {}\n", note.source));
    md.push_str(&format!("quiver_updated: {}\n", note.updated_at));
    md.push_str(&format!("quiver_path: {}\n", note.path));
    // emit any extra custom_data fields
    for (k, v) in &note.custom_data {
        if k != "uuid" && k != "quiver_file" {
            md.push_str(&format!("{k}: {v}\n"));
        }
    }
    md.push_str("---\n\n");
    md.push_str(&format!("# {}\n\n", note.title));

    for (i, cell) in note.cells.iter().enumerate() {
        let kind_str = match &cell.kind {
            CellKind::Markdown       => "markdown".to_string(),
            CellKind::Code { .. }    => "code".to_string(),
            CellKind::Text           => "text".to_string(),
            CellKind::Latex          => "latex".to_string(),
            CellKind::Diagram        => "diagram".to_string(),
            CellKind::Other(s)       => s.clone(),
        };
        md.push_str(&format!("<!-- cell: {kind_str}[{i}] -->\n"));
        let rendered = match &cell.kind {
            CellKind::Code { language } =>
                format!("```{language}\n{}\n```\n", cell.data.trim_end()),
            _ => format!("{}\n", cell.data),
        };
        if !rendered.trim().is_empty() {
            md.push_str(&rendered);
            md.push('\n');
        }
    }
    md
}

fn needs_update(old: &str, new: &str) -> bool {
    if old == new { return false; }
    TextDiff::from_lines(old, new)
        .iter_all_changes()
        .any(|c| c.tag() != ChangeTag::Equal)
}

/// Recursively collect all `.md` files under `dir` and parse them into `Note`s.
fn collect_md_notes(root: &Path, dir: &Path, out: &mut Vec<Note>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_md_notes(root, &p, out);
        } else if p.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&p) {
                if let Some(note) = parse_md_note(&content, root, &p) {
                    out.push(note);
                }
            }
        }
    }
}

/// Parse a rendered markdown file back into a `Note`.
/// Relies on the frontmatter keys and `<!-- cell: kind[i] -->` markers written by `render_note`.
pub fn parse_md_note(content: &str, root: &Path, file_path: &Path) -> Option<Note> {
    // --- split frontmatter ---
    let rest = content.strip_prefix("---\n")?;
    let (fm, body) = rest.split_once("\n---\n")?;

    // parse frontmatter key/value lines
    let mut tags: Vec<String> = Vec::new();
    let mut source = String::new();
    let mut updated_at: u64 = 0;
    let mut path = String::new();
    let mut in_tags = false;

    for line in fm.lines() {
        if line.starts_with("  - ") && in_tags {
            tags.push(line[4..].to_string());
            continue;
        }
        in_tags = false;
        if line == "tags:" { in_tags = true; continue; }
        if let Some(v) = line.strip_prefix("quiver_file: ")    { source = v.to_string(); }
        if let Some(v) = line.strip_prefix("quiver_updated: ") { updated_at = v.trim().parse().unwrap_or(0); }
        if let Some(v) = line.strip_prefix("quiver_path: ")    { path = v.to_string(); }
    }

    // derive title from the `# Title` heading (first non-blank line after frontmatter)
    let title = body.lines()
        .find(|l| l.starts_with("# "))
        .map(|l| l[2..].to_string())
        .unwrap_or_else(|| {
            file_path.file_stem().unwrap_or_default().to_string_lossy().to_string()
        });

    // if path not in frontmatter, derive from relative dir
    if path.is_empty() {
        if let Some(parent) = file_path.parent() {
            if let Ok(rel) = parent.strip_prefix(root) {
                path = rel.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");
            }
        }
    }

    // --- parse cells using <!-- cell: kind[i] --> markers ---
    let cell_marker = regex_lite_find_cells(body);
    let cells = cell_marker;

    Some(Note {
        title,
        tags,
        updated_at,
        path,
        source,
        cells,
        custom_data: HashMap::new(),
    })
}

/// Split body on `<!-- cell: kind[i] -->` markers and build `NoteCell`s.
fn regex_lite_find_cells(body: &str) -> Vec<NoteCell> {
    let mut cells = Vec::new();
    let mut remaining = body;

    loop {
        // find next marker
        let Some(marker_start) = remaining.find("<!-- cell: ") else { break };
        let marker_end = match remaining[marker_start..].find(" -->") {
            Some(o) => marker_start + o + 4,
            None => break,
        };
        let marker = &remaining[marker_start + 11..marker_end - 4]; // "kind[i]"
        let kind_str = marker.split('[').next().unwrap_or("markdown");

        // content is everything until the next marker (or end)
        let content_start = marker_end;
        let content_end = remaining[content_start..]
            .find("<!-- cell: ")
            .map(|o| content_start + o)
            .unwrap_or(remaining.len());

        let raw = remaining[content_start..content_end].trim();

        let (kind, data) = if kind_str == "code" {
            // strip fenced block: ```lang\n...\n```
            let inner = raw
                .strip_prefix("```").unwrap_or(raw);
            let lang_end = inner.find('\n').unwrap_or(0);
            let language = inner[..lang_end].to_string();
            let code = inner[lang_end..].trim_start_matches('\n');
            let code = code.strip_suffix("```").unwrap_or(code).trim_end();
            (CellKind::Code { language }, code.to_string())
        } else {
            let kind = match kind_str {
                "markdown" => CellKind::Markdown,
                "text"     => CellKind::Text,
                "latex"    => CellKind::Latex,
                "diagram"  => CellKind::Diagram,
                other      => CellKind::Other(other.to_string()),
            };
            (kind, raw.to_string())
        };

        if !data.is_empty() {
            cells.push(NoteCell { kind, data });
        }

        remaining = &remaining[content_end..];
    }

    cells
}

impl Provider for MarkdownProvider {
    fn name(&self) -> &str { "obsidian-markdown" }

    fn read_notes(&self) -> Vec<Note> {
        let mut notes = Vec::new();
        collect_md_notes(&self.out_dir, &self.out_dir, &mut notes);
        notes
    }

    fn write_note(&self, note: &Note) {
        let note_dir = self.out_dir.join(sanitize(&note.path));
        fs::create_dir_all(&note_dir).unwrap();
        let out_path = note_dir.join(format!("{}.md", sanitize(&note.title)));

        // fast-path: timestamp unchanged
        if let Some(ts) = existing_updated_at(&out_path) {
            if ts == note.updated_at { return; }
        }

        let new_md = render_note(note);

        if out_path.exists() {
            let old_md = fs::read_to_string(&out_path).unwrap_or_default();
            if !needs_update(&old_md, &new_md) { return; }
        }

        fs::write(&out_path, &new_md).unwrap();
        println!("  updated: {}", out_path.display());
    }
}
