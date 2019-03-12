use std::fs;
use std::path::Path;
use std::path::PathBuf;
use plist::Value;
use colored::*;
use rusqlite::types::ToSql;
use rusqlite::OpenFlags;
use rusqlite::{Connection, Result, NO_PARAMS};
use crypto::digest::Digest;
use crypto::sha1::Sha1;

#[macro_use]
extern crate serde;

use serde::{Deserialize};

#[derive(Debug)]
enum ClassKeyTypes {
    UUID,
    CLAS,
    WRAP,
    WPKY,
    KTYP,
    PBKY,
    VERS,
    TYPE,
    HMCK,
    SALT,
    ITER,
    DPWT,
    DPIC,
    DPSL,
    Unknown
}

impl From<&str> for ClassKeyTypes {
    fn from(string: &str) -> ClassKeyTypes {
        match string {
            "CLAS" => ClassKeyTypes::CLAS,
            "WRAP" => ClassKeyTypes::WRAP,
            "WPKY" => ClassKeyTypes::WPKY,
            "KTYP" => ClassKeyTypes::KTYP,
            "PBKY" => ClassKeyTypes::PBKY,
            "UUID" => ClassKeyTypes::UUID,
            "VERS" => ClassKeyTypes::VERS,
            "TYPE" => ClassKeyTypes::TYPE,
            "HMCK" => ClassKeyTypes::HMCK,
            "SALT" => ClassKeyTypes::SALT,
            "ITER" => ClassKeyTypes::ITER,
            "DPWT" => ClassKeyTypes::DPWT,
            "DPIC" => ClassKeyTypes::DPIC,
            "DPSL" => ClassKeyTypes::DPSL,
            x => {
                println!("unknown tag type: {}", x);
                return ClassKeyTypes::Unknown
            }
        }
    }
}

#[derive(Debug)]
enum KeybagTypes {
    System, Backup, Escrow, iCloud
}

#[derive(Debug)]
enum KeyTypes {
    Aes, Curve25519
}


/// https://stackoverflow.com/questions/1498342/how-to-decrypt-an-encrypted-apple-itunes-iphone-backup
#[derive(Debug)]
enum ProtectionClass {
    NSFileProtectionComplete,
    NSFileProtectionCompleteUnlessOpen,
    NSFileProtectionCompleteUntilFirstUserAuthentication,
    NSFileProtectionNone,
    NSFileProtectionRecovery,
    kSecAttrAccessibleWhenUnlocked,
    kSecAttrAccessibleAfterFirstUnlock,
    kSecAttrAccessibleAlways,
    kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
    kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly,
    kSecAttrAccessibleAlwaysThisDeviceOnly,
    Unknown
}

impl From<u8> for ProtectionClass {
    fn from(value: u8) -> ProtectionClass {
        match value {
            1  => ProtectionClass::NSFileProtectionComplete,
            2  => ProtectionClass::NSFileProtectionCompleteUnlessOpen,
            3  => ProtectionClass::NSFileProtectionCompleteUntilFirstUserAuthentication,
            4  => ProtectionClass::NSFileProtectionNone,
            5  => ProtectionClass::NSFileProtectionRecovery,
            6  => ProtectionClass::kSecAttrAccessibleWhenUnlocked,
            7  => ProtectionClass::kSecAttrAccessibleAfterFirstUnlock,
            8  => ProtectionClass::kSecAttrAccessibleAlways,
            9  => ProtectionClass::kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
            10 => ProtectionClass::kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly,
            11 => ProtectionClass::kSecAttrAccessibleAlwaysThisDeviceOnly,
            _  => ProtectionClass::Unknown
            
        }
    }
}

// CLASSKEY_TAGS = ["CLAS","WRAP","WPKY", "KTYP", "PBKY"]  #UUID
// KEYBAG_TYPES = ["System", "Backup", "Escrow", "OTA (icloud)"]
// KEY_TYPES = ["AES", "Curve25519"]
// PROTECTION_CLASSES={
//     1:"NSFileProtectionComplete",
//     2:"NSFileProtectionCompleteUnlessOpen",
//     3:"NSFileProtectionCompleteUntilFirstUserAuthentication",
//     4:"NSFileProtectionNone",
//     5:"NSFileProtectionRecovery?",

//     6: "kSecAttrAccessibleWhenUnlocked",
//     7: "kSecAttrAccessibleAfterFirstUnlock",
//     8: "kSecAttrAccessibleAlways",
//     9: "kSecAttrAccessibleWhenUnlockedThisDeviceOnly",
//     10: "kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly",
//     11: "kSecAttrAccessibleAlwaysThisDeviceOnly"
// }

struct BackupKeyBag {
    // _type: String,
    // uuid: String,
    // wrap: String,
    data: Vec<u8>
}

enum BackupKeyBagError {
    ParseLengthError
}

impl BackupKeyBag {
    fn get_u8_4(vec: &[u8]) -> Option<[u8; 4]> {
        if vec.len() < 4 {
            return None
        } else {
            return Some([vec[0], vec[1], vec[2], vec[3]])
        }
    }
    fn parse_tlb_blocks(&self) -> Option<Vec<BackupKeyBagBlock>> {
        let mut i = 0;
        let mut blocks = vec![];

        println!("parse tlb blocks: {}", self.data.len());
        while i + 8 < self.data.len() {
            let tag = match std::str::from_utf8(&self.data[i..i+4]) {
                Ok(res) => ClassKeyTypes::from(res),
                Err(err) => panic!("Error parsing key type: {}")
            };
            let x : [u8; 4] = match BackupKeyBag::get_u8_4(&self.data[i+4..i+8]) {
                Some(el) => el,
                None => return None
            };
            let length = u32::from_be_bytes(x) as usize;
            let data = Vec::from(&self.data[i+8..i+8+length]);

            println!("tag: {:?}, length: {}",  tag, length);
            blocks.push(BackupKeyBagBlock {
                tag, length, data
            });

            i += 8 + length;
        }

        Some(blocks)
    }
}

#[derive(Debug)]
struct BackupKeyBagBlock {
    tag: ClassKeyTypes,
    length: usize,
    data: Vec<u8>
}

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
        let info : BackupInfo = plist::from_file(format!("{}/Info.plist", path.to_str().unwrap())).unwrap();

        let manifest : BackupManifest= plist::from_file(format!("{}/Manifest.plist", path.to_str().unwrap())).unwrap();

        let status : BackupStatus = plist::from_file(format!("{}/Status.plist", path.to_str().   unwrap())).unwrap();   

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

        let conn = Connection::open_with_flags(format!("{}/Manifest.db", self.path.to_str().unwrap()), OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();

        let mut stmt = conn.prepare("SELECT * from Files")?;
        let rows = stmt.query_map(NO_PARAMS, |row| {
            // fileid equals sha1(format!("{}-{}", domain, relative_filename))
            let fileid : String = row.get(0)?;
            let domain : String = row.get(1)?;
            let relative_filename : String = row.get(2)?;

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
    #[serde(with = "serde_bytes")]
    manifest_key: Vec<u8>,

    lockdown: BackupManifestLockdown,
    #[serde(with = "serde_bytes")]
    backup_key_bag: Vec<u8>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
struct BackupManifestLockdown {
    product_version: String,
    product_type: String,
    build_version: Option<String>,
    #[serde(alias = "UniqueDeviceID")]
    unique_device_id: String,
    serial_number: String,
    device_name: String
}

#[derive(Deserialize, Debug)]
struct BackupInfo {
    #[serde(alias = "Build Version")]
    build_version: Option<String>,

    #[serde(alias = "Device Name")]
    device_name: Option<String>,

    #[serde(alias = "GUID")]
    guid: Option<String>,

    #[serde(alias = "ICCID")]
    iccid: Option<String>,

    #[serde(alias = "IMEI")]
    imei: Option<String>,

    #[serde(alias = "MEID")]
    meid: Option<String>,

    #[serde(alias = "Phone Number")]
    phone_number: Option<String>,

    #[serde(alias = "Product Type")]
    product_type: String,

    #[serde(alias = "Product Name")]
    product_name: Option<String>,

    #[serde(alias = "Product Version")]
    product_version: String,

    #[serde(alias = "Serial Number")]
    serial_number: Option<String>,

    #[serde(alias = "Target Identifier")]
    target_identifier: String,

    #[serde(alias = "Target Type")]
    target_type: String,

    #[serde(alias = "Unique Identifier")]
    unique_identifier: Option<String>,

    #[serde(alias = "iTunes Version")]
    itunes_version: Option<String>
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
            println!("{:?}", entry.path());
            let path = entry.path();
            let mut backup : Backup = Backup::new(&path).unwrap();
            println!("{:#?}", backup);
            backup.parse_manifest();
            let kb = BackupKeyBag {
                data: backup.manifest.backup_key_bag
            };

            kb.parse_tlb_blocks();
            // println!("{:#?}", );
            println!("loaded {} files from manifest", backup.files.len());
        }
    }
}



// fn print_info() {
//     let value = Value::from_file("tests/data/xml.plist").unwrap();

// }