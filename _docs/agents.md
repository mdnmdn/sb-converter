# sb-converter

A CLI tool written in Rust that converts between note formats using a provider abstraction. Currently supports [Quiver](https://yliansoft.com/) `.qvlibrary` as source and [Obsidian](https://obsidian.md/)-compatible Markdown as destination, with the architecture designed to support additional providers (online wikis, Git repos, etc.).

## Goal

Quiver stores notes as JSON-based bundles with a UUID-keyed directory structure. Obsidian expects plain `.md` files in a human-readable folder hierarchy. This tool bridges the two formats via an agnostic intermediate model, enabling bidirectional and multi-provider sync.

## Project Structure

```
sb-converter/
├── Cargo.toml
├── src/
│   ├── lib.rs                      # Crate root (re-exports modules)
│   ├── main.rs                     # CLI entry point
│   ├── model.rs                    # Agnostic Note / NoteCell / CellKind
│   ├── provider.rs                 # Provider trait
│   └── providers/
│       ├── mod.rs
│       ├── quiver.rs               # Quiver provider (read-only)
│       └── markdown.rs             # Obsidian Markdown provider (read+write)
├── tests/
│   ├── conversion.rs               # Integration + chain round-trip tests
│   └── fixtures/
│       ├── quiver/TEST.qvlibrary/  # Minimal Quiver fixture
│       └── markdown/TestNotebook/  # Expected Markdown output fixture
├── _data/
│   └── Sample-quiver.qvlibrary/   # Real sample input for development
└── _docs/
    ├── agents.md                   # This file
    ├── configuration.md            # CLI parameters, config file & profiles
    ├── quiver-format.md            # Quiver format specification
    └── guide-add-new-providers.md  # How to add a new provider
```

## Agnostic Model (`model.rs`)

All providers convert to/from this intermediate representation:

```rust
struct Note {
    title: String,
    tags: Vec<String>,
    updated_at: u64,          // Unix timestamp (0 if unknown)
    path: String,             // Logical path, e.g. "Folder/Notebook"
    source: String,           // Source identifier (library name, URL, …)
    cells: Vec<NoteCell>,
    custom_data: HashMap<String, Value>,  // Provider-specific extras
}

struct NoteCell {
    kind: CellKind,           // Markdown | Code { language } | Text | Latex | Diagram | Other
    data: String,
}
```

`custom_data` is an open-ended `HashMap<String, serde_json::Value>` for provider-specific fields (e.g. Quiver stores `uuid`; a future Git provider could store `commit_sha`).

### Structure

A **Note** is a flat document with metadata and an ordered list of **cells**. There is no notebook object in the model — the notebook hierarchy is encoded in `note.path` (e.g. `"Projects/dreamai"`). Each **NoteCell** has a `kind` and raw `data`:

| CellKind | data content |
|---|---|
| `Markdown` | GFM markdown text |
| `Code { language }` | source code; language tag preserved |
| `Text` | plain text |
| `Latex` | LaTeX / math content |
| `Diagram` | diagram source (e.g. sequence diagram) |
| `Other(String)` | unknown/future cell types |

Cells are ordered and indexed — the index is embedded in the `<!-- cell: kind[i] -->` marker in the rendered Markdown, enabling lossless round-trip parsing.

## Provider Trait (`provider.rs`)

```rust
trait Provider {
    fn name(&self) -> &str;
    fn readonly(&self) -> bool { false }   // override to true for read-only sources
    fn read_notes(&self) -> Vec<Note>;
    fn write_note(&self, note: &Note);     // panics on read-only providers
}
```

## Providers

| Provider | File | Direction | Notes |
|---|---|---|---|
| `QuiverProvider` | `providers/quiver.rs` | read-only | Parses `.qvlibrary` JSON bundles |
| `MarkdownProvider` | `providers/markdown.rs` | read + write | Renders/parses Obsidian-compatible `.md` files |

### Markdown format conventions

Each rendered `.md` file contains:
- **YAML frontmatter**: `tags`, `quiver_file`, `quiver_updated`, `quiver_path`
- **Cell markers**: `<!-- cell: kind[i] -->` before each cell, enabling lossless re-parsing
- **Differential sync**: write is skipped if `quiver_updated` timestamp matches and `similar` TextDiff confirms no content change

## How It Works

1. `main.rs` resolves config: merges config file profile → env vars → CLI flags.
2. Instantiates source and destination `Provider`s (auto-detected or explicit).
3. Calls `src.read_notes()` → `Vec<Note>`.
4. For each note calls `dst.write_note(&note)`.
5. If `--delete-missing`: computes the set of destination pages not present in source and removes them.
6. The destination provider handles differential sync internally (skips unchanged notes).

## Tech Stack

- **Language**: Rust (edition 2024)
- **Crate**: `sb-converter` (lib + bin)
- **Dependencies**: `serde`, `serde_json`, `similar` (diff)
- **Dev dependencies**: `tempfile` (test fixtures)

## Reference

- See `_docs/configuration.md` for CLI parameters, config file format, and profiles.
- See `_docs/quiver-format.md` for a full description of the Quiver file format.
- See `_docs/guide-add-new-providers.md` for instructions on adding a new provider.
