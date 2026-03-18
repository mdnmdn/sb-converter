# CLI & Configuration

## Usage

```
sb-converter [OPTIONS]                          # run a sync
sb-converter profiles                           # list all profiles
sb-converter profiles show [<name>]             # show resolved settings for a profile
sb-converter --config <file> [--profile <name>]
```

## Parameters

| Flag | Short | Env | Description |
|---|---|---|---|
| `--from <path>` | `-f` | `SBC_FROM` | Source folder or file |
| `--to <path>` | `-t` | `SBC_TO` | Destination folder |
| `--from-format <fmt>` | | `SBC_FROM_FORMAT` | Force source format (e.g. `quiver`, `markdown`). Auto-detected from path if omitted |
| `--to-format <fmt>` | | `SBC_TO_FORMAT` | Force destination format. Auto-detected if omitted |
| `--delete-missing` | `-d` | `SBC_DELETE_MISSING` | Delete destination pages that no longer exist in source |
| `--config <file>` | `-c` | `SBC_CONFIG` | Path to config file. Overrides default search order |
| `--profile <name>` | `-p` | `SBC_PROFILE` | Named sync profile to use (default: `default`) |

## Profile Commands

```
sb-converter profiles
```
Lists all profile names found in the resolved config file, marking the default.

```
sb-converter profiles show [<name>]
```
Prints the fully-resolved settings for `<name>` (or `default` if omitted), after merging env vars and CLI flags. Useful for debugging what would actually run.

## Format Auto-detection

If `--from-format` / `--to-format` are not specified, the format is inferred from the path:

| Pattern | Detected format |
|---|---|
| `*.qvlibrary` | `quiver` |
| directory (no special extension) | `markdown` |

## Configuration File

Supported formats: **TOML** or **YAML**. Searched in order (first found wins):

1. Path given by `--config` / `SBC_CONFIG`
2. `./sb-converter.toml` or `./sb-converter.yaml` (current directory)
3. `~/.config/sb-converter/config.toml` or `~/.config/sb-converter/config.yaml`

```toml
# sb-converter.toml

[default]
from = "_data/Sample-quiver.qvlibrary"
to   = "_output"
delete_missing = false

[profiles.work]
from = "/path/to/work.qvlibrary"
to   = "/path/to/obsidian-vault"
delete_missing = true

[profiles.archive]
from           = "/path/to/archive.qvlibrary"
to             = "/path/to/archive-vault"
from_format    = "quiver"
to_format      = "markdown"
delete_missing = false
```

Equivalent YAML:

```yaml
# sb-converter.yaml  (or ~/.config/sb-converter/config.yaml)

default:
  from: _data/Sample-quiver.qvlibrary
  to: _output
  delete_missing: false

profiles:
  work:
    from: /path/to/work.qvlibrary
    to: /path/to/obsidian-vault
    delete_missing: true

  archive:
    from: /path/to/archive.qvlibrary
    to: /path/to/archive-vault
    from_format: quiver
    to_format: markdown
    delete_missing: false
```

## Precedence (highest → lowest)

1. CLI flags
2. Environment variables
3. Config file profile (`--profile`, default: `default`)
4. Built-in defaults

## Examples

```bash
# Auto-detect formats, use defaults
sb-converter --from my.qvlibrary --to ./vault

# Force formats explicitly
sb-converter -f ./notes -t ./vault --from-format markdown --to-format markdown

# Delete pages missing from source
sb-converter -f my.qvlibrary -t ./vault --delete-missing

# Use a named profile from config
sb-converter --profile work

# Use a custom config file and profile
sb-converter --config ~/sync.toml --profile archive

# List all profiles in the resolved config
sb-converter profiles

# Show resolved settings for the 'work' profile
sb-converter profiles show work
```
