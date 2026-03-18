# Quiver Format

Quiver stores its data as a `.qvlibrary` directory (a macOS package/bundle).

## Directory Structure

```
MyLibrary.qvlibrary/
├── meta.json                          # Library structure (folder tree)
├── Inbox.qvnotebook/                  # Special built-in notebook
│   ├── meta.json
│   └── <UUID>.qvnote/
│       ├── meta.json
│       └── content.json
├── Trash.qvnotebook/                  # Special built-in notebook
│   └── ...
└── <UUID>.qvnotebook/                 # Regular notebook
    ├── meta.json
    └── <UUID>.qvnote/
        ├── meta.json
        ├── content.json
        └── resources/                 # Optional: attached files
```

## Library `meta.json`

Defines the **recursive folder/notebook tree**. Notebooks can be nested inside other notebooks acting as folders. The tree is defined by `children` arrays.

```json
{
  "uuid": "Notebooks",
  "children": [
    { "uuid": "08B60D32-..." },
    {
      "uuid": "39082FCA-...",
      "children": [
        { "uuid": "480F89CA-..." },
        { "uuid": "CBE3A139-..." }
      ]
    },
    {
      "uuid": "B4C9DA88-...",
      "children": [
        {
          "uuid": "BB231508-...",
          "children": [
            { "uuid": "3A46F11E-..." }
          ]
        }
      ]
    }
  ]
}
```

- A node with no `children` key is a leaf notebook.
- A node with `children` acts as a folder containing other notebooks (and can itself also be a notebook with notes).
- The nesting can be arbitrarily deep.

## Notebook `meta.json`

```json
{
  "name": "Triade",
  "uuid": "785F704D-4402-4B17-BA6D-13B13B0D0839"
}
```

Fields: `name` (display name), `uuid`.

## Note `meta.json`

```json
{
  "created_at": 1713367227,
  "updated_at": 1713368127,
  "tags": ["tag1", "tag2"],
  "title": "Note Title",
  "uuid": "8F05213D-120E-48F2-B8CF-B3183611EF0E"
}
```

Fields:
- `title` — note title
- `uuid` — unique identifier
- `created_at` / `updated_at` — Unix timestamps
- `tags` — array of strings (may be empty)

## Note `content.json`

A note is composed of an ordered array of **cells**. Each cell has a `type` and `data`.

```json
{
  "title": "Note Title",
  "cells": [
    {
      "type": "markdown",
      "data": "# Heading\n\nSome **markdown** content."
    },
    {
      "type": "code",
      "language": "sql",
      "data": "SELECT * FROM table;"
    }
  ]
}
```

### Cell Types

| Type       | Description                                      | Extra fields       |
|------------|--------------------------------------------------|--------------------|
| `markdown` | Markdown text (GFM)                              | —                  |
| `code`     | Code snippet with syntax highlighting            | `language` (string)|
| `text`     | Plain text                                       | —                  |
| `latex`    | LaTeX / math content                             | —                  |
| `diagram`  | Diagram (e.g. sequence diagram)                  | —                  |

In practice, most notes use `markdown` and `code` cells. Multiple cells of the same type can appear in sequence.

## Resources

A note may have a `resources/` subdirectory containing attached files (images, PDFs, etc.). These are referenced from cell content via relative paths like `quiver-image-url://resources/<filename>`.

## Conversion to Obsidian

| Quiver concept          | Obsidian equivalent                        |
|-------------------------|--------------------------------------------|
| Library folder tree     | Nested folder structure                    |
| Notebook name           | Folder name                                |
| Note title              | File name (`<title>.md`)                   |
| `markdown` cell         | Appended as-is to the `.md` file           |
| `code` cell             | Wrapped in fenced code block with language |
| `tags` in meta          | YAML frontmatter `tags:`                   |
| `created_at/updated_at` | YAML frontmatter dates (optional)          |
| `resources/` files      | Copied to an attachments folder            |
| Trash notebook          | Skipped                                    |
