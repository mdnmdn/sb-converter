# sb-converter — Second Brain Converter

A CLI tool written in Rust that converts between note formats. Currently supports [Quiver](https://yliansoft.com/) `.qvlibrary` as source and [Obsidian](https://obsidian.md/)-compatible Markdown as destination.

## Quick Start

```bash
# Convert a Quiver library to Obsidian Markdown
sb-converter --from my.qvlibrary --to ./vault

# Use a named profile from config
sb-converter --profile work

# List available profiles
sb-converter profiles
```

## Installation

```bash
cargo build --release
# binary at: target/release/sb-converter
```

## Configuration

Supports TOML or YAML config files. Default lookup order:

1. `--config <file>` / `SBC_CONFIG`
2. `./sb-converter.toml` or `./sb-converter.yaml`
3. `~/.config/sb-converter/config.toml` or `~/.config/sb-converter/config.yaml`

See [`sb-converter.toml`](sb-converter.toml) for a sample config with named profiles.

Full CLI reference: [`_docs/configuration.md`](_docs/configuration.md)

## Supported Formats

| Format | Read | Write |
|---|---|---|
| Quiver (`.qvlibrary`) | ✅ | ✅ |
| Obsidian Markdown | ✅ | ✅ |

## License

[MIT](LICENSE)
