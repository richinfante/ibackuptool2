#[derive(Debug, PartialEq)]
pub enum KeybagTypes {
    System, Backup, Escrow, iCloud, Unknown
}

impl From<u32> for KeybagTypes {
    fn from(value: u32) -> KeybagTypes {
        match value {
            0 => KeybagTypes::System,
            1 => KeybagTypes::Backup,
            2 => KeybagTypes::Escrow,
            3 => KeybagTypes::iCloud,
            _ => KeybagTypes::Unknown
        }
    }
}