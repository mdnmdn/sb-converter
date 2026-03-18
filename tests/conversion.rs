use std::collections::HashMap;
use sb_converter::{
    model::{CellKind, Note, NoteCell},
    provider::Provider,
    providers::{
        markdown::{MarkdownProvider, parse_md_note, render_note},
        quiver::QuiverProvider,
    },
};

fn sample_note() -> Note {
    Note {
        title: "Test Note".into(),
        tags: vec!["rust".into(), "test".into()],
        updated_at: 1713368127,
        path: "TestNotebook".into(),
        source: "TEST.qvlibrary".into(),
        cells: vec![
            NoteCell { kind: CellKind::Markdown, data: "# Hello\n\nFirst cell.".into() },
            NoteCell { kind: CellKind::Code { language: "rust".into() }, data: "fn main() {}".into() },
            NoteCell { kind: CellKind::Markdown, data: "Second markdown cell.".into() },
        ],
        custom_data: HashMap::new(),
    }
}

// --- model tests ---

#[test]
fn note_has_correct_cell_count() {
    assert_eq!(sample_note().cells.len(), 3);
}

#[test]
fn code_cell_has_language() {
    let note = sample_note();
    assert!(matches!(&note.cells[1].kind, CellKind::Code { language } if language == "rust"));
}

// --- quiver provider tests ---

#[test]
fn quiver_reads_notes() {
    let provider = QuiverProvider::new("tests/fixtures/quiver/TEST.qvlibrary");
    let notes = provider.read_notes();
    assert_eq!(notes.len(), 1);
    let note = &notes[0];
    assert_eq!(note.title, "Test Note");
    assert_eq!(note.tags, vec!["rust", "test"]);
    assert_eq!(note.updated_at, 1713368127);
    assert_eq!(note.path, "TestNotebook");
    assert_eq!(note.cells.len(), 3);
}

#[test]
fn quiver_is_not_readonly() {
    let provider = QuiverProvider::new("tests/fixtures/quiver/TEST.qvlibrary");
    assert!(!provider.readonly());
}

#[test]
fn quiver_custom_data_has_uuid() {
    let provider = QuiverProvider::new("tests/fixtures/quiver/TEST.qvlibrary");
    let notes = provider.read_notes();
    assert_eq!(notes[0].custom_data["uuid"], "NOTE-UUID");
}

// --- quiver write tests ---

#[test]
fn quiver_write_creates_note_files() {
    let tmp = tempfile::tempdir().unwrap();
    let lib = tmp.path().join("OUT.qvlibrary");
    let provider = QuiverProvider::new(&lib);
    provider.write_note(&sample_note());

    // find the single .qvnote directory
    let nb = std::fs::read_dir(&lib).unwrap()
        .flatten()
        .find(|e| e.path().extension().and_then(|x| x.to_str()) == Some("qvnotebook"))
        .expect("notebook dir");
    let note_dir = std::fs::read_dir(nb.path()).unwrap()
        .flatten()
        .find(|e| e.path().extension().and_then(|x| x.to_str()) == Some("qvnote"))
        .expect("note dir");

    assert!(note_dir.path().join("meta.json").exists());
    assert!(note_dir.path().join("content.json").exists());
}

#[test]
fn quiver_write_meta_fields() {
    let tmp = tempfile::tempdir().unwrap();
    let lib = tmp.path().join("OUT.qvlibrary");
    let provider = QuiverProvider::new(&lib);
    provider.write_note(&sample_note());

    let nb = std::fs::read_dir(&lib).unwrap().flatten()
        .find(|e| e.path().extension().and_then(|x| x.to_str()) == Some("qvnotebook")).unwrap();
    let note_dir = std::fs::read_dir(nb.path()).unwrap().flatten()
        .find(|e| e.path().extension().and_then(|x| x.to_str()) == Some("qvnote")).unwrap();

    let meta: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(note_dir.path().join("meta.json")).unwrap()
    ).unwrap();
    assert_eq!(meta["title"], "Test Note");
    assert_eq!(meta["updated_at"], 1713368127u64);
    assert_eq!(meta["tags"][0], "rust");
}

#[test]
fn quiver_write_content_cells() {
    let tmp = tempfile::tempdir().unwrap();
    let lib = tmp.path().join("OUT.qvlibrary");
    let provider = QuiverProvider::new(&lib);
    provider.write_note(&sample_note());

    let nb = std::fs::read_dir(&lib).unwrap().flatten()
        .find(|e| e.path().extension().and_then(|x| x.to_str()) == Some("qvnotebook")).unwrap();
    let note_dir = std::fs::read_dir(nb.path()).unwrap().flatten()
        .find(|e| e.path().extension().and_then(|x| x.to_str()) == Some("qvnote")).unwrap();

    let content: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(note_dir.path().join("content.json")).unwrap()
    ).unwrap();
    let cells = content["cells"].as_array().unwrap();
    assert_eq!(cells.len(), 3);
    assert_eq!(cells[1]["type"], "code");
    assert_eq!(cells[1]["language"], "rust");
    assert_eq!(cells[1]["data"], "fn main() {}");
}

#[test]
fn quiver_write_skips_unchanged() {
    let tmp = tempfile::tempdir().unwrap();
    let lib = tmp.path().join("OUT.qvlibrary");
    let provider = QuiverProvider::new(&lib);
    let note = sample_note();
    provider.write_note(&note);

    // find meta.json and record mtime
    let nb = std::fs::read_dir(&lib).unwrap().flatten()
        .find(|e| e.path().extension().and_then(|x| x.to_str()) == Some("qvnotebook")).unwrap();
    let note_dir = std::fs::read_dir(nb.path()).unwrap().flatten()
        .find(|e| e.path().extension().and_then(|x| x.to_str()) == Some("qvnote")).unwrap();
    let meta_path = note_dir.path().join("meta.json");
    let mtime1 = std::fs::metadata(&meta_path).unwrap().modified().unwrap();

    std::thread::sleep(std::time::Duration::from_millis(10));
    provider.write_note(&note);
    let mtime2 = std::fs::metadata(&meta_path).unwrap().modified().unwrap();
    assert_eq!(mtime1, mtime2, "meta.json should not be rewritten when unchanged");
}

#[test]
fn quiver_write_then_read_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    let lib = tmp.path().join("OUT.qvlibrary");
    let note = sample_note();

    let writer = QuiverProvider::new(&lib);
    writer.write_note(&note);

    let reader = QuiverProvider::new(&lib);
    let notes = reader.read_notes();
    assert_eq!(notes.len(), 1);
    let r = &notes[0];
    assert_eq!(r.title, note.title);
    assert_eq!(r.tags, note.tags);
    assert_eq!(r.updated_at, note.updated_at);
    assert_eq!(r.cells.len(), note.cells.len());
    for (o, w) in note.cells.iter().zip(r.cells.iter()) {
        assert_eq!(std::mem::discriminant(&o.kind), std::mem::discriminant(&w.kind));
        assert_eq!(o.data, w.data);
    }
}

// --- markdown render tests ---

#[test]
fn render_contains_frontmatter() {
    let md = render_note(&sample_note());
    assert!(md.contains("quiver_updated: 1713368127"));
    assert!(md.contains("quiver_path: TestNotebook"));
    assert!(md.contains("quiver_file: TEST.qvlibrary"));
    assert!(md.contains("tags:"));
    assert!(md.contains("  - rust"));
}

#[test]
fn render_contains_cell_markers() {
    let md = render_note(&sample_note());
    assert!(md.contains("<!-- cell: markdown[0] -->"));
    assert!(md.contains("<!-- cell: code[1] -->"));
    assert!(md.contains("<!-- cell: markdown[2] -->"));
}

#[test]
fn render_code_cell_fenced() {
    let md = render_note(&sample_note());
    assert!(md.contains("```rust\nfn main() {}\n```"));
}

#[test]
fn render_matches_fixture() {
    let md = render_note(&sample_note());
    let expected = std::fs::read_to_string("tests/fixtures/markdown/TestNotebook/Test Note.md").unwrap();
    assert_eq!(md, expected);
}

// --- markdown provider write tests ---

#[test]
fn markdown_write_creates_file() {
    let tmp = tempfile::tempdir().unwrap();
    let provider = MarkdownProvider::new(tmp.path());
    provider.write_note(&sample_note());
    let out = tmp.path().join("TestNotebook").join("Test Note.md");
    assert!(out.exists());
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("quiver_updated: 1713368127"));
}

#[test]
fn markdown_write_skips_unchanged() {
    let tmp = tempfile::tempdir().unwrap();
    let provider = MarkdownProvider::new(tmp.path());
    let note = sample_note();
    provider.write_note(&note);
    let out = tmp.path().join("TestNotebook").join("Test Note.md");
    let mtime1 = std::fs::metadata(&out).unwrap().modified().unwrap();
    // small sleep to ensure mtime would differ if re-written
    std::thread::sleep(std::time::Duration::from_millis(10));
    provider.write_note(&note);
    let mtime2 = std::fs::metadata(&out).unwrap().modified().unwrap();
    assert_eq!(mtime1, mtime2, "file should not be rewritten when unchanged");
}

#[test]
fn markdown_is_not_readonly() {
    let provider = MarkdownProvider::new("_output");
    assert!(!provider.readonly());
}

// --- round-trip: quiver → agnostic → markdown ---

#[test]
fn quiver_to_markdown_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    let src = QuiverProvider::new("tests/fixtures/quiver/TEST.qvlibrary");
    let dst = MarkdownProvider::new(tmp.path());
    for note in src.read_notes() {
        dst.write_note(&note);
    }
    let out = tmp.path().join("TestNotebook").join("Test Note.md");
    assert!(out.exists());
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("# Test Note"));
    assert!(content.contains("<!-- cell: code[1] -->"));
}

// --- chain: qu → md → qu ---
// Quiver is read-only so "back to qu" means: parse the written markdown and
// compare the agnostic Note fields against the original.

#[test]
fn chain_qu_md_qu() {
    let tmp = tempfile::tempdir().unwrap();

    // qu → md
    let src = QuiverProvider::new("tests/fixtures/quiver/TEST.qvlibrary");
    let dst = MarkdownProvider::new(tmp.path());
    let original_notes = src.read_notes();
    for note in &original_notes {
        dst.write_note(note);
    }

    // md → qu (via MarkdownProvider::read_notes)
    let recovered = dst.read_notes();

    assert_eq!(recovered.len(), original_notes.len(), "note count must survive round-trip");
    let orig = &original_notes[0];
    let rec  = &recovered[0];

    assert_eq!(rec.title,      orig.title);
    assert_eq!(rec.tags,       orig.tags);
    assert_eq!(rec.updated_at, orig.updated_at);
    assert_eq!(rec.path,       orig.path);
    assert_eq!(rec.cells.len(), orig.cells.len(), "cell count must survive round-trip");

    for (o, r) in orig.cells.iter().zip(rec.cells.iter()) {
        assert_eq!(
            std::mem::discriminant(&o.kind),
            std::mem::discriminant(&r.kind),
            "cell kind must survive round-trip"
        );
        assert_eq!(o.data, r.data, "cell data must survive round-trip");
    }
}

// --- chain: md → qu → md ---
// Start from the fixture markdown, parse it, re-render it, compare strings.

#[test]
fn chain_md_qu_md() {
    let fixture_path = std::path::Path::new("tests/fixtures/markdown/TestNotebook/Test Note.md");
    let fixture_root = std::path::Path::new("tests/fixtures/markdown");
    let original_md = std::fs::read_to_string(fixture_path).unwrap();

    // md → agnostic Note
    let note = parse_md_note(&original_md, fixture_root, fixture_path)
        .expect("fixture must parse");

    assert_eq!(note.title,      "Test Note");
    assert_eq!(note.tags,       vec!["rust", "test"]);
    assert_eq!(note.updated_at, 1713368127);
    assert_eq!(note.path,       "TestNotebook");
    assert_eq!(note.cells.len(), 3);

    // agnostic Note → md
    let re_rendered = render_note(&note);

    // The re-rendered output must equal the original fixture exactly.
    assert_eq!(re_rendered, original_md, "md→note→md must be identity");
}
