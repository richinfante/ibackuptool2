#![allow(non_camel_case_types)]

/// https://stackoverflow.com/questions/1498342/how-to-decrypt-an-encrypted-apple-itunes-iphone-backup
#[derive(Debug, PartialEq)]
pub enum ProtectionClass {
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

impl From<u32> for ProtectionClass {
    fn from(value: u32) -> ProtectionClass {
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