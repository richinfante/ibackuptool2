#[derive(Debug, PartialEq)]
pub enum KeyTypes {
    Aes, Curve25519, Unknown
}

impl From<u32> for KeyTypes {
    fn from(value: u32) -> KeyTypes {
        match value {
            0 => KeyTypes::Aes,
            1 => KeyTypes::Curve25519,
            _ => KeyTypes::Unknown
        }
    }
}