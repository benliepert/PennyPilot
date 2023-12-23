use crate::entry::Entry;

use csv::ReaderBuilder;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Result as IoResult, Write};
use std::path::PathBuf;

pub fn write_entries_to_csv(entries: &[Entry], file_path: &PathBuf) -> IoResult<()> {
    let backup_path = file_path.with_extension("backup");
    File::create(&backup_path)?;
    let dest_path = file_path.clone();
    // dest_path.set_file_name(backup_path);
    std::fs::copy(file_path, dest_path)?;

    // opens the file if it exists and clears its contents, which is why we create a backup
    let mut file = File::create(file_path)?;

    debug!("Writing entries");
    for entry in entries {
        let csv_string = entry.to_csv_string();
        writeln!(file, "{}", csv_string)?;
    }

    if backup_path.exists() {
        // everything worked, remove the backup file now
        // fs::remove_file(backup_path)?;
        debug!("Removing backup file");
    }

    Ok(())
}

/// Read entries from something implementing the `Read` trait
fn read_entries_from_reader<R: Read>(reader: R) -> Result<Vec<Entry>, Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().has_headers(false).from_reader(reader);

    rdr.records()
        .filter_map(|result| result.ok())
        // Entry implements try_from<StringRecord> to make this simple for us
        .map(Entry::try_from)
        .collect()
}

/// Read a vector of `Entry`s from a csv file at `file_path`
pub fn read_entries_from_file(file_path: &PathBuf) -> Result<Vec<Entry>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    read_entries_from_reader(file)
}

#[cfg(target_arch = "wasm32")]
/// Read a vector of `Entry`s from a byte vector
pub fn read_entries_from_vec(buffer: Vec<u8>) -> Result<Vec<Entry>, Box<dyn Error>> {
    use std::io::Cursor;
    let cursor = Cursor::new(buffer);
    read_entries_from_reader(cursor)
}
