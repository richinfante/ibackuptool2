use crate::lib::crypto::*;
use super::file::{BackupFile, FileInfo};
use super::info::BackupInfo;
use super::manifest::{BackupManifest, BackupManifestLockdown};
use super::status::BackupStatus;
use super::Backup;

use std::convert::TryFrom;
use std::io::Read;
use std::path::{Path};

use rusqlite::OpenFlags;
use rusqlite::{Connection, NO_PARAMS};

use std::cell::RefCell;
use zip::{self, ZipArchive};


pub fn parse_manifest<'a>(backup: &mut Backup<'a>) -> Result<(), Box<dyn std::error::Error>> {
    debug!("parsing manifest!");
    let conn: Connection;
    let tmpf = tempfile::TempDir::new()?;

    let decpath = tmpf.path().join("manifest.db");

    // let contents = backup.raw_file_read("Manifest.mbdb").unwrap();
    
    // debug!("{:?}", mbdb_manifest::parse_manifest(contents));
    // panic!();

    {
        if backup.manifest.is_encrypted {
            let contents = backup.raw_file_read("Manifest.db")?;

            // let path = format!("{}/Manifest.db", self.path.to_str().unwrap());
            // let contents = std::fs::read(Path::new(&path)).unwrap();
            let decrypted_db = crate::lib::crypto::decrypt_with_key(
                &backup.manifest.manifest_key_unwrapped.as_ref().unwrap(),
                &contents,
            );
            debug!("decrypted {} bytes from manifest.", decrypted_db.len());

            trace!("writing decrypted database: {}", decpath.display());
            // let decpath = Path::new(&pth);
            std::fs::write(&decpath, decrypted_db)?;

            // NOTE:
            // this is opened read write.
            // I have *no idea* why readonly does this, but it failes every time with "cannot open databsse", code 14.
            // since this is a copy, it's read write
            conn = Connection::open_with_flags(&decpath, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
            trace!("wrote decrypted database to tmp: {}", decpath.display());

            // std::thread::sleep(std::time::Duration::from_secs(15));
        } else {
            conn = Connection::open_with_flags(
                format!("{}/Manifest.db", backup.path.to_str().unwrap()),
                OpenFlags::SQLITE_OPEN_READ_ONLY,
            )?;
        }

        let mut stmt =
            conn.prepare("SELECT fileid, domain, relativePath, flags, file from Files")?;
        let rows = stmt
            .query_map(NO_PARAMS, |row| {
                // fileid equals sha1(format!("{}-{}", domain, relative_filename))
                let fileid: String = row.get(0)?;
                let domain: String = row.get(1)?;
                let relative_filename: String = row.get(2)?;
                let flags: i64 = row.get(3)?;
                let file: Vec<u8> = row.get(4)?;
                use plist::Value;

                let cur = std::io::Cursor::new(file);
                let val = Value::from_reader(cur).expect("expected to load bplist");

                let fileinfo = match FileInfo::try_from(val) {
                    Ok(res) => Some(res),
                    Err(err) => {
                        error!("failed to parse file info: {}", err);
                        None
                    }
                };

                Ok(BackupFile {
                    fileid,
                    domain,
                    relative_filename,
                    flags,
                    fileinfo,
                })
            })
            .expect("Query to succeed");

        // Add each item to the internal list
        for item in rows {
            if let Ok(item) = item {
                backup.files.push(item);
            }
        }
    }

    tmpf.close()?;

    Ok(())

}