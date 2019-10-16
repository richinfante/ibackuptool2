use log::warn;

#[derive(Debug, PartialEq, Clone)]
pub enum KeybagBlockTag {
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
    Unknown,
}

impl From<&str> for KeybagBlockTag {
    fn from(string: &str) -> KeybagBlockTag {
        match string {
            "CLAS" => KeybagBlockTag::CLAS,
            "WRAP" => KeybagBlockTag::WRAP,
            "WPKY" => KeybagBlockTag::WPKY,
            "KTYP" => KeybagBlockTag::KTYP,
            "PBKY" => KeybagBlockTag::PBKY,
            "UUID" => KeybagBlockTag::UUID,
            "VERS" => KeybagBlockTag::VERS,
            "TYPE" => KeybagBlockTag::TYPE,
            "HMCK" => KeybagBlockTag::HMCK,
            "SALT" => KeybagBlockTag::SALT,
            "ITER" => KeybagBlockTag::ITER,
            "DPWT" => KeybagBlockTag::DPWT,
            "DPIC" => KeybagBlockTag::DPIC,
            "DPSL" => KeybagBlockTag::DPSL,
            x => {
                warn!("unknown tag type: {}", x);
                return KeybagBlockTag::Unknown;
            }
        }
    }
}
