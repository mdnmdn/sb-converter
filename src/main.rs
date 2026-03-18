use sb_converter::{
    diff::diff_paths,
    provider::Provider,
    providers::{markdown::MarkdownProvider, quiver::QuiverProvider},
};
use std::{env, path::Path};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("diff") => {
            let (a, b) = match (args.get(2), args.get(3)) {
                (Some(a), Some(b)) => (a.as_str(), b.as_str()),
                _ => {
                    eprintln!("usage: sb-converter diff <a> <b>");
                    std::process::exit(1);
                }
            };
            diff_paths(Path::new(a), Path::new(b));
        }
        _ => {
            let src = QuiverProvider::new("_data/Sample-quiver.qvlibrary");
            let dst = MarkdownProvider::new("_output");

            let notes = src.read_notes();
            println!("Read {} notes from '{}'", notes.len(), src.name());

            for note in &notes {
                dst.write_note(note);
            }

            println!("Done → {}", Path::new("_output").display());
        }
    }
}
