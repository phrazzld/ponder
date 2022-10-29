use std::process::Command;
use std::fs::OpenOptions;
use std::io::{Error, Read, Write};
use chrono;

fn main() -> Result<(), Error> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    let now = chrono::Local::now();

    let filename = format!("{}/rubberducks/{}.md", std::env::var("HOME").unwrap(), now.format("%Y%m%d"));
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(true)
        .open(filename)?;

    // If file is empty, append today's date in YYYY MMM, DD format as the header
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    if contents.is_empty() {
        writeln!(file, "# {}\n", now.format("%B %d, %Y: %A"))?;
    }

    // Append the current timestamp as a subheader
    writeln!(file, "## {}\n\n", now.format("%H:%M:%S"))?;

    // Shadow filename to get around issue with using filename after moving it to file
    let filename = format!("{}/rubberducks/{}.md", std::env::var("HOME").unwrap(), now.format("%Y%m%d"));

    // Open today's rubberduck with $EDITOR
    Command::new(editor)
        .arg(filename)
        .status()
        .expect("Failed to open file");

    Ok(())
}

// TODO: Add automated tests
// TODO: Add CLI configuration options
// TODO: Support encryption
