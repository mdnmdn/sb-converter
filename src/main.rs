use sb_converter::{
    providers::{quiver::QuiverProvider, markdown::MarkdownProvider},
    provider::Provider,
};
use std::path::Path;

fn main() {
    let src = QuiverProvider::new("_data/Sample-quiver.qvlibrary");
    let dst = MarkdownProvider::new("_output");

    let notes = src.read_notes();
    println!("Read {} notes from '{}'", notes.len(), src.name());

    for note in &notes {
        dst.write_note(note);
    }

    println!("Done → {}", Path::new("_output").display());
}
