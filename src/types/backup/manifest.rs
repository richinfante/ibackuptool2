
use serde::{Deserialize};
use serde_bytes::ByteBuf;
use crate::types::*;

#[derive(Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
pub struct BackupManifest {
    pub is_encrypted: bool,
    pub version: String,
    pub date: String,
    pub system_domains_version: String,
    pub was_passcode_set: bool,
    pub manifest_key: Option<ByteBuf>,
    pub lockdown: BackupManifestLockdown,
    pub backup_key_bag: Option<ByteBuf>,

    #[serde(skip)]
    pub keybag: Option<KeyBag>,

    #[serde(skip)]
    pub manifest_key_unwrapped: Option<Vec<u8>>
}


// impl BackupManifest {
//   fn unlock_manifest(&mut self, key: &Vec<u8>) {
//     if let Some(ref manifest_key) = self.manifest_key {
//       let manifest_wrapped : Vec<u8> = manifest_key.;
//       let result_key = crate::types::crypto::unwrap_key(key.as_slice(), &manifest_wrapped);
//       self.manifest_key_unwrapped = Some(result_key);
//     }
//   }
// }

#[derive(Deserialize, Debug)]
#[serde(rename_all="PascalCase")]
pub struct BackupManifestLockdown {
    pub product_version: String,
    pub product_type: String,
    pub build_version: Option<String>,
    #[serde(alias = "UniqueDeviceID")]
    pub unique_device_id: String,
    pub serial_number: String,
    pub device_name: String,
}
