mod cli;
mod config;
mod editor;
mod errors;
mod journal;

use cli::parse_args;
use config::Config;
use editor::{Editor, SystemEditor};
use errors::AppResult;
use journal::io::FileSystemIO;
use journal::Journal;
use log::info;

fn main() -> AppResult<()> {
    env_logger::init();
    info!("Starting ponder");

    let args = parse_args();
    let config = Config::load()?;
    config.validate()?;
    config.ensure_journal_dir()?;
    
    let io = FileSystemIO {
        journal_dir: config.journal_dir.to_string_lossy().to_string(),
    };
    let journal = Journal::new(io);
    
    let editor = SystemEditor {
        editor_cmd: config.editor,
    };

    if args.reminisce {
        let paths = journal.get_reminisce_entries()?;
        if paths.is_empty() {
            println!("No entries found for reminisce intervals");
            return Ok(());
        }
        editor.open_files(&paths)?;
    } else if args.retro {
        let paths = journal.get_retro_entries()?;
        if paths.is_empty() {
            println!("No entries found for the past week");
            return Ok(());
        }
        editor.open_files(&paths)?;
    } else {
        let path = journal.get_todays_entry_path()?;
        journal.append_date_time(&path)?;
        editor.open_files(&[path])?;
    }

    Ok(())
}