use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct BackupStatus {
    pub backup_state: String,
    pub date: String,
    pub is_full_backup: bool,
    pub snapshot_state: String,
    #[serde(alias = "UUID")]
    pub uuid: String,
    pub version: String,
}
