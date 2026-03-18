# Adding a New Provider

1. Create `src/providers/<name>.rs` implementing the `Provider` trait.
2. Map the source format's concepts to `Note` / `NoteCell` / `CellKind`.
3. Store any format-specific fields in `note.custom_data`.
4. Register in `src/providers/mod.rs`.

## Read-only vs Read-Write

- Override `readonly() -> bool { true }` if the source cannot be written back to.
- `write_note` has a default impl that panics — read-only providers don't need to implement it.

## Example skeleton

```rust
pub struct MyProvider { /* config */ }

impl Provider for MyProvider {
    fn name(&self) -> &str { "my-provider" }
    fn readonly(&self) -> bool { true }

    fn read_notes(&self) -> Vec<Note> {
        // fetch / parse source, map to Vec<Note>
        todo!()
    }
}
```

## custom_data conventions

Use `note.custom_data` for fields that have no equivalent in the agnostic model:

```rust
custom_data.insert("my_id".into(), json!("abc-123"));
```

Consumers of the agnostic model will ignore unknown keys, so this is safe across providers.
