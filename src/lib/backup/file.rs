use crate::lib::*;
use ::plist::Value;

#[derive(Debug, Clone)]
/// Holds file metadata
/// This corresponds to the `file` field of the manifest database
pub struct FileInfo {
    /// File last modified date
    pub last_modified: u64,

    /// Last status change
    pub last_status_change: u64,

    /// File creation date
    pub birth: u64,

    /// File flags
    pub flags: u64,

    /// Filesystem inode number
    pub inode: u64,

    /// Filesystem owner group id
    pub group_id: u64,

    /// Filesystem owner user id
    pub user_id: u64,

    /// File size
    pub size: u64,

    /// File mode
    pub mode: u64,

    /// File protection class used for key wrapping
    pub protection_class: ProtectionClass,

    /// Wrapped version of the file encryption key
    pub wrapped_encryption_key: Option<Vec<u8>>,
    pub wrapped_encryption_class: Option<ProtectionClass>,

    /// Unwrapped encryption key
    pub encryption_key: Option<Vec<u8>>,

    /// File extended attributes
    pub extended_attributes: Option<Vec<u8>>,
}

impl FileInfo {
    pub fn unwrap_encryption_key(&mut self, keybag: &KeyBag) {
        // guard wrapped key
        let wrapped_encryption_key = match &self.wrapped_encryption_key {
            Some(el) => el,
            _ => return,
        };

        // guard class key
        let class_key = match keybag.find_class_key(&self.protection_class) {
            Some(class_key) => class_key,
            _ => return,
        };

        let result_key =
            crate::lib::crypto::unwrap_key(&class_key.as_slice(), wrapped_encryption_key);
        self.encryption_key = Some(result_key);
    }
}

#[derive(Debug, Clone)]
pub struct BackupFile {
    /// This corresponds to the hash of the file id on disk.
    /// fileid equals sha1(format!("{}-{}", domain, relative_filename))
    pub fileid: String,

    /// This corresponds to the domain the file is contained in.
    /// example: "MediaDomain" or "CameraRollDomain"
    pub domain: String,

    /// The file path, relative to the domain
    pub relative_filename: String,

    // File flags
    pub flags: i64,

    pub fileinfo: Option<FileInfo>,
}

impl BackupFile {
    pub fn unwrap_file_key(&mut self, backup: &Backup) {
        let keybag = match backup.get_keybag() {
            Some(kb) => kb,
            None => return,
        };

        match self.fileinfo.as_mut() {
            Some(fileinfo) => {
                fileinfo.unwrap_encryption_key(keybag);
            }
            None => {}
        }
    }
}

use std::convert::TryFrom;
impl TryFrom<::plist::Value> for FileInfo {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: ::plist::Value) -> Result<FileInfo, Self::Error> {
        // First, decode as an NSKeyedArchiver archive.
        let fork = crate::lib::plist::decode_nskeyedarchiver(value);

        if let Value::Dictionary(mut forkdict) = fork {
            // Unwrap contained binaryy data / attributes
            let val = forkdict.remove("EncryptionKey");
            if let Some(Value::Dictionary(dict)) = val {
                if let Some(Value::Data(data)) = dict.get("NS.data") {
                    let protclass = as_u32_le(&data[0..4]);
                    let mankey = &data[4..];

                    forkdict.insert("EncryptionKey".to_string(), Value::Data(mankey.to_vec()));
                    forkdict.insert(
                        "EncryptionKeyClass".to_string(),
                        Value::Integer(::plist::Integer::from(protclass)),
                    );
                }
            }

            let exta = forkdict.remove("ExtendedAttributes");
            if let Some(Value::Dictionary(dict)) = exta {
                if let Some(Value::Data(data)) = dict.get("NS.data") {
                    forkdict.insert("ExtendedAttributes".to_string(), Value::Data(data.to_vec()));
                }
            }

            /// read and unwrap a contained uint inside of a dict
            fn read_uint(key: &str, dict: &::plist::Dictionary) -> Option<u64> {
                if let Some(val) = dict.get(key) {
                    return val.as_unsigned_integer();
                }

                None
            }

            /// read and unwrap, clone a contained Vec<u8>
            fn read_data(key: &str, dict: &::plist::Dictionary) -> Option<Vec<u8>> {
                if let Some(Value::Data(data)) = dict.get(key) {
                    return Some(data.clone());
                }

                None
            }

            return Ok(FileInfo {
                // modification times
                last_modified: read_uint("LastModified", &forkdict).unwrap_or(0),
                last_status_change: read_uint("LastStatusChange", &forkdict).unwrap_or(0),
                birth: read_uint("Birth", &forkdict).unwrap_or(0),

                // filesystem info
                inode: read_uint("InodeNumber", &forkdict).unwrap_or(0),
                user_id: read_uint("UserID", &forkdict).unwrap_or(0),
                group_id: read_uint("GroupID", &forkdict).unwrap_or(0),
                mode: read_uint("Mode", &forkdict).unwrap_or(0),
                size: read_uint("Size", &forkdict).unwrap_or(0),
                flags: read_uint("Flags", &forkdict).unwrap_or(0),

                // encryption + extensions
                protection_class: ProtectionClass::from(
                    read_uint("ProtectionClass", &forkdict).unwrap_or(99) as u32,
                ),
                wrapped_encryption_class: Some(ProtectionClass::from(
                    read_uint("EncryptionKeyClass", &forkdict).unwrap_or(99) as u32,
                )),
                wrapped_encryption_key: read_data("EncryptionKey", &forkdict),
                encryption_key: None,
                extended_attributes: read_data("ExtendedAttributes", &forkdict),
            });
        }

        unimplemented!()
    }
}
