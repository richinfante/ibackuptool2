mod manifest;
mod info;
mod status;
mod file;

pub use manifest::{BackupManifest, BackupManifestLockdown};
pub use info::{BackupInfo};
pub use status::BackupStatus;
pub use file::BackupFile;
use crate::lib::crypto::*;

use std::path::Path;

use rusqlite::OpenFlags;
use rusqlite::{Connection, NO_PARAMS};

#[derive(Debug)]
pub struct Backup<'a> {
    pub path: Box<&'a Path>,
    pub manifest: BackupManifest,
    pub info: BackupInfo,
    pub status: BackupStatus,
    pub files: Vec<BackupFile>
}

impl Backup<'_> {
    /// Create from root backup path.
    pub fn new(path: &Path) -> Result<Backup, Box<dyn std::error::Error>> {
        let status : BackupStatus = plist::from_file(format!("{}/Status.plist", path.to_str().unwrap()))?;

        let info : BackupInfo = plist::from_file(format!("{}/Info.plist", path.to_str().unwrap()))?;

        let manifest : BackupManifest= plist::from_file(format!("{}/Manifest.plist", path.to_str().unwrap()))?;

        Ok(Backup {
            path: Box::new(path.clone()),
            manifest,
            status,
            info,
            files: vec![]
        })
    }

    /// Parse the keybag contained in the manifest.
    pub fn parse_keybag(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(bag) = &self.manifest.backup_key_bag {
            self.manifest.keybag = Some(KeyBag::init(bag.to_vec())); 
        }

        Ok(())
    }

    /// Load the list of files, from the backup's manifest file.
    pub fn parse_manifest(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.files.clear();

        let conn : Connection;
        if self.manifest.is_encrypted {
            let path = format!("{}/Manifest.db", self.path.to_str().unwrap());
            let contents = std::fs::read(Path::new(&path)).unwrap();
            let dec = crate::lib::crypto::decrypt_with_key(&self.manifest.manifest_key_unwrapped.as_ref().unwrap(), &contents);
            debug!("decrypted {} bytes from manifest.", dec.len());
            let home_dir = match dirs::home_dir() {
                Some(res) => match res.to_str() {
                    Some(res) => res.to_string(),
                    None => panic!("Can't convert homedir to string!")
                },
                None => panic!("Can't find homedir:")
            };

            let pth = format!("{}/Downloads/decrypted_database.sqlite", home_dir);
            trace!("writing decrypted database: {}", pth);
            let decpath = Path::new(&pth);
            std::fs::write(&decpath, dec).unwrap();

            conn = Connection::open_with_flags(&decpath, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        } else {
            conn = Connection::open_with_flags(format!("{}/Manifest.db", self.path.to_str().unwrap()), OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        }

        let mut stmt = conn.prepare("SELECT * from Files")?;
        let rows = stmt.query_map(NO_PARAMS, |row| {
            // fileid equals sha1(format!("{}-{}", domain, relative_filename))
            let fileid : String = row.get(0)?;
            let domain : String = row.get(1)?;
            let relative_filename : String = row.get(2)?;
            let flags : i64 = row.get(3)?;

            Ok(BackupFile {
                fileid,
                domain,
                relative_filename,
                flags
            })
        })?;

        // Add each item to the internal list
        for item in rows {
            if let Ok(item) = item {
                self.files.push(item);
            }
        }

        Ok(())
        
    }
}
