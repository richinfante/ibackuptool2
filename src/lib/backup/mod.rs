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
use std::io::Read;
use std::path::{Path};

use rusqlite::OpenFlags;
use rusqlite::{Connection, NO_PARAMS};

use std::cell::RefCell;
use zip::{self, ZipArchive};

/// Stores where this backup is physcially stored.
/// Internal retreival can change depending on if this an actual file or a zip file
#[derive(Debug)]
pub enum BackupBacking {
    Filesystem,
    ZipFile(RefCell<ZipArchive<std::fs::File>>),
}

#[derive(Debug)]
pub struct Backup<'a> {
    pub path: Box<&'a Path>,
    pub manifest: BackupManifest,
    pub info: BackupInfo,
    pub status: BackupStatus,
    pub files: Vec<BackupFile>,
    pub relative_root: Option<String>,
    pub backing: BackupBacking,
}

fn read_archive_file(
    archive: &mut zip::ZipArchive<std::fs::File>,
    path: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut res: Vec<u8> = vec![];
    archive.by_name(path)?.read_to_end(&mut res)?;

    Ok(res)
}

impl Backup<'_> {
    /// Create from root backup path.
    pub fn new(path: &Path) -> Result<Backup, Box<dyn std::error::Error>> {
        let status;
        let manifest;
        let info;
        let mut relative_root: Option<String> = None;
        let mut backing = BackupBacking::Filesystem;

        if path.is_file() && path.extension().and_then(std::ffi::OsStr::to_str) == Some("zip") {
            let mut archive = zip::ZipArchive::new(std::fs::File::open(path)?)?;

            let names = archive
                .file_names()
                .map(|v| v.to_string())
                .collect::<Vec<String>>();
            let mut root_path: Option<String> = None;

            for name in names {
                if name.ends_with("Manifest.plist") {
                    println!("{}", name);
                    let path = Path::new(&name);
                    root_path = path
                        .parent()
                        .and_then(|v| v.as_os_str().to_str().and_then(|v| Some(v.to_string())));
                    debug!("found zip root: {:?}", path.parent());
                    break;
                }
            }

            let zip_root = match root_path {
            Some(v) => v,
            None => panic!("could not find the Manifest.plist inside the zip file. Is this actually a backup?")
          };

            status = plist::from_bytes(&read_archive_file(
                &mut archive,
                &format!("{}/Status.plist", &zip_root),
            )?)?;
            info = plist::from_bytes(&read_archive_file(
                &mut archive,
                &format!("{}/Info.plist", &zip_root),
            )?)?;
            manifest = plist::from_bytes(&read_archive_file(
                &mut archive,
                &format!("{}/Manifest.plist", &zip_root),
            )?)?;

            // init ctrl vars
            relative_root = Some(zip_root);
            backing = BackupBacking::ZipFile(RefCell::new(archive));
        } else {
            status = plist::from_file(format!("{}/Status.plist", path.to_str().unwrap()))?;
            info = plist::from_file(format!("{}/Info.plist", path.to_str().unwrap()))?;
            manifest = plist::from_file(format!("{}/Manifest.plist", path.to_str().unwrap()))?;
        }

        Ok(Backup {
            path: Box::new(path.clone()),
            manifest,
            status,
            info,
            relative_root,
            files: vec![],
            backing,
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

    pub fn raw_file_read(&self, path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match &self.backing {
            BackupBacking::Filesystem => {
                // prepend fs path
                let finpath = self.path.join(Path::new(&path));
                if !finpath.is_file() {
                    return Err(crate::lib::error::BackupError::InManifestButNotFound.into());
                }

                Ok(std::fs::read(&finpath)?)
            }
            BackupBacking::ZipFile(archive) => read_archive_file(
                &mut archive.borrow_mut(),
                &match &self.relative_root {
                    Some(rootpath) => format!("{}/{}", rootpath, path),
                    None => path.to_string(),
                },
            ),
        }
    }

    #[allow(dead_code)]
    pub fn read_file(&self, file: &BackupFile) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let path = format!(
            "{}/{}/{}",
            self.path.to_str().expect("path to be str"),
            (&file.fileid)[0..2].to_string(),
            file.fileid
        );

        debug!("read backup file path: {}", path);

        let contents = self.raw_file_read(&path)?;

        // if the file
        if self.manifest.is_encrypted {
            debug!("file {} is encrypted, decrypting...", path);
            match &file.fileinfo.as_ref() {
                Some(fileinfo) => match fileinfo.encryption_key.as_ref() {
                    Some(encryption_key) => {
                        let dec = crate::lib::crypto::decrypt_with_key(&encryption_key, &contents);
                        debug!("file {} is now decrypted...", path);
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
                let contents = self.raw_file_read("Manifest.db")?;

                // let path = format!("{}/Manifest.db", self.path.to_str().unwrap());
                // let contents = std::fs::read(Path::new(&path)).unwrap();
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
                    self.files.push(item);
                }
            }
        }

        tmpf.close()?;

        Ok(())
    }
}
