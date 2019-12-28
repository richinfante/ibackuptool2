pub mod address;
pub mod outputformat;
pub mod sms;

pub use address::*;
pub use outputformat::*;
pub use sms::*;

use crate::lib::*;
use rusqlite::Connection;
use std::io::Write;

pub struct SqliteProxy {
    pub domain: String,
    pub path: String,
    connection: Connection,
    tmpfile: tempfile::NamedTempFile,
}

impl SqliteProxy {
    pub fn new(
        backup: &Backup,
        domain: &str,
        path: &str,
    ) -> Result<SqliteProxy, Box<dyn std::error::Error>> {
        let mut file = match backup.find_path(domain, path) {
            Some(file) => file,
            None => return Err(crate::lib::BackupError::FileNotFound.into()),
        };

        if backup.manifest.is_encrypted {
            file.unwrap_file_key(backup);
        }
        let mut tmpfile = tempfile::NamedTempFile::new()?;
        
        trace!("sqliteproxy: read {:?} into {}", file, tmpfile.path().display());
        tmpfile.write(backup.read_file(&file).expect("read to succeed").as_slice())?;

        let connection = Connection::open(tmpfile.path())?;

        Ok(SqliteProxy {
            domain: domain.to_string(),
            path: path.to_string(),
            connection,
            tmpfile,
        })
    }
}
