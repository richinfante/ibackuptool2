use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct BackupInfo {
    #[serde(alias = "Build Version")]
    pub build_version: Option<String>,

    #[serde(alias = "Device Name")]
    pub device_name: Option<String>,

    #[serde(alias = "GUID")]
    pub guid: Option<String>,

    #[serde(alias = "ICCID")]
    pub iccid: Option<String>,

    #[serde(alias = "IMEI")]
    pub imei: Option<String>,

    #[serde(alias = "MEID")]
    pub meid: Option<String>,

    #[serde(alias = "Phone Number")]
    pub phone_number: Option<String>,

    #[serde(alias = "Product Type")]
    pub product_type: String,

    #[serde(alias = "Product Name")]
    pub product_name: Option<String>,

    #[serde(alias = "Product Version")]
    pub product_version: String,

    #[serde(alias = "Serial Number")]
    pub serial_number: Option<String>,

    #[serde(alias = "Target Identifier")]
    pub target_identifier: String,

    #[serde(alias = "Target Type")]
    pub target_type: String,

    #[serde(alias = "Unique Identifier")]
    pub unique_identifier: Option<String>,

    #[serde(alias = "iTunes Version")]
    pub itunes_version: Option<String>,
}
