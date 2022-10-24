mod file;
mod info;
mod manifest;
mod status;

use crate::lib::crypto::*;
pub use file::{BackupFile, FileInfo};
pub use info::BackupInfo;
pub use manifest::{BackupManifest, BackupManifestLockdown};
pub use status::BackupStatus;

use std::convert::TryFrom;
use std::path::Path;

use rusqlite::OpenFlags;
use rusqlite::{Connection, NO_PARAMS};

#[derive(Debug)]
pub struct Backup<'a> {
    pub path: Box<&'a Path>,
    pub manifest: BackupManifest,
    pub info: BackupInfo,
    pub status: BackupStatus,
    pub files: Vec<BackupFile>,
}

impl Backup<'_> {
    /// Create from root backup path.
    pub fn new(path: &Path) -> Result<Backup, Box<dyn std::error::Error>> {
        let status: BackupStatus =
            plist::from_file(format!("{}/Status.plist", path.to_str().unwrap()))?;

        let info: BackupInfo = plist::from_file(format!("{}/Info.plist", path.to_str().unwrap()))?;

        let manifest: BackupManifest =
            plist::from_file(format!("{}/Manifest.plist", path.to_str().unwrap()))?;

        Ok(Backup {
            path: Box::new(path.clone()),
            manifest,
            status,
            info,
            files: vec![],
        })
    }

    /// Parse the keybag contained in the manifest.
    pub fn parse_keybag(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(bag) = &self.manifest.backup_key_bag {
            self.manifest.keybag = Some(KeyBag::init(bag.to_vec()));
        }

        Ok(())
    }

    pub fn get_keybag(&self) -> Option<&KeyBag> {
        match &self.manifest.keybag {
            Some(kb) => Some(kb),
            None => return None,
        }
    }

    #[allow(dead_code)]
    pub fn find_fileid(&self, fileid: &str) -> Option<BackupFile> {
        for file in &self.files {
            if file.fileid == fileid {
                return Some(file.clone());
            }
        }

        return None;
    }

    #[allow(dead_code)]
    pub fn find_path(&self, domain: &str, path: &str) -> Option<BackupFile> {
        for file in &self.files {
            if file.relative_filename == path && file.domain == domain {
                return Some(file.clone());
            }
        }

        return None;
    }

    #[allow(dead_code)]
    pub fn read_file(&self, file: &BackupFile) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let path = format!(
            "{}/{}/{}",
            self.path.to_str().expect("path to be str"),
            (&file.fileid)[0..2].to_string(),
            file.fileid
        );
        let finpath = self.path.join(Path::new(&path));

        debug!("read file path: {}", finpath.display());

        if !finpath.is_file() {
            return Err(crate::lib::error::BackupError::InManifestButNotFound.into());
        }

        let contents = std::fs::read(&finpath).expect("contents to exist");

        // if the file
        if self.manifest.is_encrypted {
            debug!("file {} is encrypted, decrypting...", finpath.display());
            match &file.fileinfo.as_ref() {
                Some(fileinfo) => match fileinfo.encryption_key.as_ref() {
                    Some(encryption_key) => {
                        let dec = crate::lib::crypto::decrypt_with_key(&encryption_key, &contents);
                        debug!("file {} is now decrypted...", finpath.display());
                        return Ok(dec);
                    }
                    None => {
                        return Err(crate::lib::error::BackupError::NoEncryptionKey.into());
                    }
                },
                None => {
                    return Err(crate::lib::error::BackupError::NoFileInfo.into());
                }
            }
        }

        return match std::fs::read(Path::new(&path)) {
            Ok(vec) => Ok(vec),
            Err(err) => Err(err.into()),
        };
    }

    /// Unwrap all individual file encryption keys
    pub fn unwrap_file_keys(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let keybag = match &self.manifest.keybag {
            Some(kb) => kb,
            None => return Ok(()),
        };

        info!("unwrapping file keys...");
        for file in self.files.iter_mut() {
            if file.fileinfo.is_some() {
                let mutable = file.fileinfo.as_mut();
                mutable.map(|s| s.unwrap_encryption_key(keybag));
            }
        }
        info!("unwrapping file keys... [done]");

        Ok(())
    }

    /// Load the list of files, from the backup's manifest file.
    pub fn parse_manifest(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.files.clear();

        let conn: Connection;
        let tmpf = tempfile::TempDir::new()?;
        let decpath = tmpf.path().join("manifest.db");

        {
          if self.manifest.is_encrypted {
              let path = format!("{}/Manifest.db", self.path.to_str().unwrap());
              let contents = std::fs::read(Path::new(&path)).unwrap();
              let decrypted_db = crate::lib::crypto::decrypt_with_key(
                  &self.manifest.manifest_key_unwrapped.as_ref().unwrap(),
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
                  format!("{}/Manifest.db", self.path.to_str().unwrap()),
                  OpenFlags::SQLITE_OPEN_READ_ONLY,
              )?;
          }

          let mut stmt =
              conn.prepare("SELECT fileid, domain, relativePath, flags, file from Files")?;
          let rows = stmt.query_map(NO_PARAMS, |row| {
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
          }).expect("Query to succeed");

          // Add each item to the internal list
          for item in rows {
              if let Ok(item) = item {
                  self.files.push(item);
              }
          }
        }

        tmpf.close()?;

        Ok(())
    }
}
