use chrono::prelude::*;
use clap::{App, Arg};
use std::env;
use std::fs::OpenOptions;
use std::io::{Error, Read, Write};
use std::process::Command;

// TODO: add cli configuration options
// TODO: support encryption
// TODO: add tests
// TODO: add reminisce command for reviewing distant entries

fn main() -> Result<(), Error> {
    let matches = App::new("ponder")
        .arg(
            Arg::with_name("retro")
                .short('r')
                .long("retro")
                .help("Opens entries from the past week excluding today"),
        )
        .get_matches();

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    if matches.is_present("retro") {
        // retrieve entries from the past week and open each
        let mut filenames = Vec::new();
        for i in (1..=7).rev() {
            let date = Local::now() - chrono::Duration::days(i);
            let filename = generate_filename_for_date(date);
            if std::fs::metadata(&filename).is_ok() {
                filenames.push(filename);
            }
        }
        if !filenames.is_empty() {
            Command::new(&editor)
                .args(&filenames)
                .status()
                .expect("Failed to open files");
        }
    } else {
        let filename = generate_filename_for_date(Local::now());
        let mut file = create_or_open_file(&filename).unwrap();
        append_date_time(&mut file).unwrap();
        Command::new(editor)
            .arg(filename)
            .status()
            .expect("Failed to open file");
    }

    Ok(())
}

fn generate_filename_for_date(date: DateTime<Local>) -> String {
    format!(
        "{}/Documents/rubberducks/{}.md",
        env::var("HOME").unwrap(),
        date.format("%Y%m%d")
    )
}

fn create_or_open_file(filename: &String) -> Result<std::fs::File, Error> {
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
        writeln!(
            file,
            "\n#journal [[{} {}]] [[year {}]]",
            now.format("%B").to_string().to_lowercase(),
            now.format("%Y"),
            now.format("%Y")
        )?;
        writeln!(file, "\n## {}\n\n", now.format("%H:%M:%S"))?;
    } else {
        writeln!(file, "\n\n## {}\n\n", now.format("%H:%M:%S"))?;
    }

    Ok(())
}
