use std::process::Command;
use std::fs::OpenOptions;
use std::io::{Error, Read, Write};
use chrono;

// TODO: Add CLI configuration options
// TODO: Support encryption

fn main() -> Result<(), Error> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    // let now = chrono::Local::now();

    // let filename = format!("{}/rubberducks/{}.md", std::env::var("HOME").unwrap(), now.format("%Y%m%d"));
    // let mut file = OpenOptions::new()
    //     .read(true)
    //     .write(true)
    //     .create(true)
    //     .append(true)
    //     .open(filename)?;
    let mut file = create_or_open_file().unwrap();

    // If file is empty, append today's date in YYYY MMM, DD format as the header
    // let mut contents = String::new();
    // file.read_to_string(&mut contents)?;
    // if contents.is_empty() {
    //     writeln!(file, "# {}", now.format("%B %d, %Y: %A"))?;
    //     writeln!(file, "\n## {}\n\n", now.format("%H:%M:%S"))?;
    // } else {
    //     writeln!(file, "\n\n## {}\n\n", now.format("%H:%M:%S"))?;
    // }
    append_date_time(&mut file).unwrap();

    // Shadow filename to get around issue with using filename after moving it to file
    let filename = generate_filename();

    // Open today's rubberduck with $EDITOR
    Command::new(editor)
        .arg(filename)
        .status()
        .expect("Failed to open file");

    Ok(())
}

fn generate_filename() -> String {
    let now = chrono::Local::now();
    format!("{}/rubberducks/{}.md", std::env::var("HOME").unwrap(), now.format("%Y%m%d"))
}

fn create_or_open_file() -> Result<std::fs::File, Error> {
    let filename = generate_filename();
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(true)
        .open(filename)?;
    Ok(file)
}

fn append_date_time(file: &mut std::fs::File) -> Result<(), Error> {
    let now = chrono::Local::now();
    // If file is empty, append today's date in YYYY MMM, DD format as the header
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    if contents.is_empty() {
        writeln!(file, "# {}", now.format("%B %d, %Y: %A"))?;
        writeln!(file, "\n## {}\n\n", now.format("%H:%M:%S"))?;
    } else {
        writeln!(file, "\n\n## {}\n\n", now.format("%H:%M:%S"))?;
    }
    Ok(())
}

#[test]
fn test_filename_generation() {
    let now = chrono::Local::now();
    let expected_filename = format!("{}/rubberducks/{}.md", std::env::var("HOME").unwrap(), now.format("%Y%m%d"));
    let filename = generate_filename();
    assert_eq!(filename, expected_filename);
}

#[test]
fn test_file_created_if_not_exist() {
    let now = chrono::Local::now();
    let filename = format!("{}/rubberducks/{}.md", std::env::var("HOME").unwrap(), now.format("%Y%m%d"));
    assert!(std::fs::metadata(filename).is_ok());
}

#[test]
fn test_date_and_time_appended_correctly() {
    let now = chrono::Local::now();
    let expected_date_time = now.format("%B %d, %Y: %A").to_string();
    let expected_time = now.format("%H:%M:%S").to_string();

    let mut file = create_or_open_file().unwrap();
    append_date_time(&mut file).unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    assert!(contents.contains(&expected_date_time));
    assert!(contents.contains(&expected_time));
}
