use std::fs;
use std::path::Path;
use std::path::PathBuf;
use plist::Value;
use colored::*;
use rusqlite::types::ToSql;
use rusqlite::{Connection, Result, NO_PARAMS};
use crypto::digest::Digest;
use crypto::sha1::Sha1;

#[macro_use]
extern crate serde;

use serde::{Deserialize};

#[derive(Debug)]
struct Backup<'a> {
    path: Box<&'a Path>,
    manifest: BackupManifest,
    info: BackupInfo,
    status: BackupStatus,
    files: Vec<BackupFile>
}

#[derive(Debug)]
struct BackupFile {
    fileid: String,
    domain: String,
    relative_filename: String
}

#[derive(Debug)]
enum BackupParseError {
    ManifestParseFailed,
    StatusParseFailed,
    InfoParseFailed
}

impl Backup<'_> {
    fn new(path: &Path) -> Result<Backup> {
        let info :BackupInfo = plist::from_file(format!("{}/Info.plist", path.to_str().unwrap())).unwrap();

        let manifest :BackupManifest = plist::from_file(format!("{}/Manifest.plist", path.to_str().unwrap())).unwrap();

        let status :BackupStatus = plist::from_file(format!("{}/Status.plist", path.to_str().unwrap())).unwrap();

        Ok(Backup {
            path: Box::new(path.clone()),
            manifest,
            status,
            info,
            files: vec![]
        })
    }

    fn parse_manifest(&mut self) -> Result<()> {
        self.files.clear();

        let conn = Connection::open(format!("{}/Manifest.mbdb", self.path.to_str().unwrap())).unwrap();

        let mut stmt = conn.prepare("SELECT * from Files")?;
        let rows = stmt.query_map(NO_PARAMS, |row| {

            let fileid : String = row.get(0)?;
            let domain : String = row.get(1)?;
            let relative_filename : String = row.get(2)?;
            // create a Sha1 object
            let mut hasher = Sha1::new();

            // write input message
            hasher.input_str(&format!("{}-{}", domain, relative_filename));

            // read hash digest
            let hex = hasher.result_str();

            println!("{} -> {}", fileid, hex);

            Ok(BackupFile {
                fileid,
                domain,
                relative_filename
            })
        })?;

        for item in rows {
            if let Ok(item) = item {
                self.files.push(item);
            }
        }

        Ok(())
        
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
struct BackupStatus {
    backup_state: String,
    date: String,
    is_full_backup: bool,
    snapshot_state: String,
    #[serde(alias = "UUID")]
    uuid: String,
    version: String
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
struct BackupManifest {
    is_encrypted: bool,
    version: String,
    date: String,
    system_domains_version: String,
    was_passcode_set: bool,
    lockdown: BackupManifestLockdown
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
struct BackupManifestLockdown {
    product_version: String,
    product_type: String,
    build_version: String,
    #[serde(alias = "UniqueDeviceID")]
    unique_device_id: String,
    serial_number: String,
    device_name: String
}

#[derive(Deserialize, Debug)]
struct BackupInfo {
    #[serde(alias = "Build Version")]
    build_version: String,

    #[serde(alias = "Device Name")]
    device_name: String,

    #[serde(alias = "GUID")]
    guid: String,

    #[serde(alias = "ICCID")]
    iccid: String,

    #[serde(alias = "IMEI")]
    imei: String,

    #[serde(alias = "MEID")]
    meid: String,

    #[serde(alias = "Phone Number")]
    phone_number: String,

    #[serde(alias = "Product Type")]
    product_type: String,

    #[serde(alias = "Product Name")]
    product_name: String,

    #[serde(alias = "Product Version")]
    product_version: String,

    #[serde(alias = "Serial Number")]
    serial_number: String,

    #[serde(alias = "Target Identifier")]
    target_identifier: String,

    #[serde(alias = "Target Type")]
    target_type: String,

    #[serde(alias = "Unique Identifier")]
    unique_identifier: String,

    #[serde(alias = "iTunes Version")]
    itunes_version: String
}

const BACKUP_DIRECTORY : &'static str = "/Library/Application Support/MobileSync/Backup/";

fn main() {
    let home_dir = match dirs::home_dir() {
        Some(res) => match res.to_str() {
            Some(res) => res.to_string(),
            None => panic!("Can't convert homedir to string!")
        },
        None => panic!("Can't find homedir:")
    };

    let backup_dir = format!("{}{}",
        home_dir,
        BACKUP_DIRECTORY
    );
    
    println!("using directory: {}", backup_dir);
    let dir = Path::new(&backup_dir);

    if dir.is_dir() {
        println!("exists!");
    }

    let ls = std::fs::read_dir(dir).unwrap();

    for entry in ls {
        let entry = entry.unwrap();
        if entry.path().is_dir() {
            let path = entry.path();
            let mut backup : Backup = Backup::new(&path).unwrap();
            backup.parse_manifest();
            println!("{:#?}", backup);
        }
    }
}



// fn print_info() {
//     let value = Value::from_file("tests/data/xml.plist").unwrap();

// }