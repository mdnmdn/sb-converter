use similar::{ChangeTag, TextDiff};
use std::{collections::HashMap, fs, path::Path};

use crate::providers::markdown::{parse_md_note, render_note};

/// A single file-level diff result.
#[derive(Debug)]
pub struct FileDiff {
    pub path: String,
    pub status: DiffStatus,
    /// Unified-diff text (empty for Added/Removed).
    pub unified: String,
}

#[derive(Debug, PartialEq)]
pub enum DiffStatus {
    Equal,
    Changed,
    Added,   // present in `b` only
    Removed, // present in `a` only
}

/// Diff two markdown strings, normalising them through the Note model so that
/// cosmetic differences (missing cell markers, merged markdown cells, …) are
/// ignored.  Returns a unified-diff string; empty means no semantic difference.
pub fn diff_md(a: &str, b: &str, label: &str) -> String {
    let dummy = Path::new("");
    let note_a = parse_md_note(a, dummy, dummy)
        .map(|n| render_note(&n))
        .unwrap_or_else(|| a.to_string());
    let note_b = parse_md_note(b, dummy, dummy)
        .map(|n| render_note(&n))
        .unwrap_or_else(|| b.to_string());

    if note_a == note_b {
        return String::new();
    }

    let diff = TextDiff::from_lines(&note_a, &note_b);
    let mut out = format!("--- {label} (a)\n+++ {label} (b)\n");
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => '-',
            ChangeTag::Insert => '+',
            ChangeTag::Equal => ' ',
        };
        out.push(sign);
        out.push_str(change.as_str().unwrap_or(""));
    }
    out
}

/// Diff two files or directories.  Prints results to stdout.
pub fn diff_paths(a: &Path, b: &Path) {
    let diffs = if a.is_file() && b.is_file() {
        vec![diff_files(a, b, &a.display().to_string())]
    } else {
        diff_dirs(a, b)
    };

    let mut any = false;
    for d in &diffs {
        match d.status {
            DiffStatus::Equal => {}
            DiffStatus::Added => {
                println!("+ {}", d.path);
                any = true;
            }
            DiffStatus::Removed => {
                println!("- {}", d.path);
                any = true;
            }
            DiffStatus::Changed => {
                println!("{}", d.unified);
                any = true;
            }
        }
    }
    if !any {
        println!("No differences.");
    }
}

fn diff_files(a: &Path, b: &Path, label: &str) -> FileDiff {
    let text_a = fs::read_to_string(a).unwrap_or_default();
    let text_b = fs::read_to_string(b).unwrap_or_default();
    let unified = diff_md(&text_a, &text_b, label);
    FileDiff {
        path: label.to_string(),
        status: if unified.is_empty() {
            DiffStatus::Equal
        } else {
            DiffStatus::Changed
        },
        unified,
    }
}

/// Collect relative `.md` paths from a directory.
fn md_files(dir: &Path) -> HashMap<String, std::path::PathBuf> {
    let mut map = HashMap::new();
    collect(dir, dir, &mut map);
    map
}

fn collect(root: &Path, dir: &Path, out: &mut HashMap<String, std::path::PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect(root, &p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("md") {
            let rel = p
                .strip_prefix(root)
                .unwrap_or(&p)
                .to_string_lossy()
                .to_string();
            out.insert(rel, p);
        }
    }
}

fn diff_dirs(a: &Path, b: &Path) -> Vec<FileDiff> {
    let files_a = md_files(a);
    let files_b = md_files(b);
    let mut keys: Vec<_> = files_a
        .keys()
        .chain(files_b.keys())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .cloned()
        .collect();
    keys.sort();

    keys.into_iter()
        .map(|rel| match (files_a.get(&rel), files_b.get(&rel)) {
            (Some(pa), Some(pb)) => diff_files(pa, pb, &rel),
            (None, Some(_)) => FileDiff {
                path: rel,
                status: DiffStatus::Added,
                unified: String::new(),
            },
            (Some(_), None) => FileDiff {
                path: rel,
                status: DiffStatus::Removed,
                unified: String::new(),
            },
            (None, None) => unreachable!(),
        })
        .collect()
}
