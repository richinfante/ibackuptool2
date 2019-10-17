use log::debug;

use crate::lib::*;
use serde::Deserialize;
use serde_bytes::ByteBuf;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
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
    pub manifest_key_unwrapped: Option<Vec<u8>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct BackupManifestLockdown {
    pub product_version: String,
    pub product_type: String,
    pub build_version: Option<String>,
    #[serde(alias = "UniqueDeviceID")]
    pub unique_device_id: String,
    pub serial_number: String,
    pub device_name: String,
}

impl BackupManifest {
    /// Unwrap manifest key using protection class
    /// https://stackoverflow.com/questions/1498342/how-to-decrypt-an-encrypted-apple-itunes-iphone-backup/13793043
    pub fn unlock_manifest(&mut self) {
        if let Some(ref manifest_key) = self.manifest_key {
            debug!("unwrapping manifest key...");
            let sliced: &Vec<u8> = manifest_key.as_ref();
            let protclass = as_u32_le(&sliced[0..4]);
            let mankey = &sliced[4..];
            debug!("manifest protection class: {:x?}", protclass);
            let clazz = ProtectionClass::from(protclass);
            let class_key = self
                .keybag
                .as_ref()
                .expect("expect locked manifest to have keybag")
                .find_class_key(&clazz)
                .unwrap();
            let items: Vec<u8> = mankey.iter().cloned().collect();
            let result_key = crate::lib::crypto::unwrap_key(&class_key, &items);
            self.manifest_key_unwrapped = Some(result_key);
            trace!("unwrapped manifest key: {:x?}", self.manifest_key_unwrapped);
            debug!("unwrapped manifest key successfully!");
        }
    }
}
