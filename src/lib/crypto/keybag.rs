
use log::{debug};

use uuid::Uuid;
use hex;

use ring::{digest, pbkdf2};
use crate::lib::crypto::*;


#[derive(Debug)]
pub struct KeyBag {
    pub kind: KeybagTypes,
    pub version: u32,
    pub uuid: Uuid,
    pub hmck: Vec<u8>,
    pub salt: Vec<u8>,
    pub double_protection_salt: Option<Vec<u8>>,
    pub iterations: Option<u32>,
    pub dpwt: Option<u32>,
    pub dpic: Option<u32>,
    pub wrap: u32,
    pub keys: Vec<KeybagEntry>,
    pub key: Option<Vec<u8>>
}

#[derive(Debug)]
pub struct KeybagEntry {
    pub uuid: Uuid,
    pub class: ProtectionClass,
    pub wrap: u32,
    pub key_type: KeyTypes,
    pub wpky: Vec<u8>,
    pub key: Option<Vec<u8>>
}

#[derive(Debug, Clone)]
pub struct BackupKeyBagBlock {
    tag: KeybagBlockTag,
    length: usize,
    data: Vec<u8>
}
    
impl KeyBag {

    fn find_contained_blocks(blocks: &Vec<BackupKeyBagBlock>) -> Vec<Vec<BackupKeyBagBlock>> {
        let mut found_uuid_count = 0;
        let mut sections : Vec<Vec<BackupKeyBagBlock>> = vec![];
        let mut section : Vec<BackupKeyBagBlock> = vec![];
        for block in blocks {
            if block.tag == KeybagBlockTag::UUID {
                found_uuid_count += 1;

               if found_uuid_count > 2 && section.len() > 0 {
                    sections.push(section.clone());
                    section.clear();
                }
            }

            if found_uuid_count > 1 {
                section.push(block.clone());
            }
        }

        sections
    }

    fn find_root_blocks(blocks: &Vec<BackupKeyBagBlock>) -> Vec<BackupKeyBagBlock> {
        let mut found_uuid = false;
        let mut root_entries = vec![];
        for block in blocks {
            if block.tag == KeybagBlockTag::UUID {
                if !found_uuid {
                    found_uuid = true;
                } else {
                    break;
                }
            }
        
            root_entries.push(block.clone());
        }

        return root_entries;
    }

    pub fn unlock_with_key(&mut self, passcode_key: Vec<u8>) {
        self.key = Some(passcode_key.clone());

        for key in self.keys.iter_mut() {
            // let k = key.wpky;
            key.key = Some(crate::lib::crypto::aes::unwrap_key(&passcode_key, &key.wpky));   
        }

        println!("unwrapped {} keys.", self.keys.len());
        for key in &self.keys {
            let classid : u32 = key.class.into();
            match key.key {
                Some(ref unwrapped) => trace!("{}: {:?} - {:?} ({}) - {}", key.uuid, key.key_type, key.class, classid, hex::encode(unwrapped)),
                None => trace!("{}: {:?} - {:?} ({}) - <none>", key.uuid, key.key_type, key.class, classid)
            }
        }

        //     def unlockWithPasscode(self, passcode, passcode_key=None):
        // if passcode_key is None:
        //     passcode1 = fastpbkdf2.pbkdf2_hmac('sha256', passcode,
        //                                     self.attrs["DPSL"],
        //                                     self.attrs["DPIC"], 32)
        //     passcode_key = fastpbkdf2.pbkdf2_hmac('sha1', passcode1,
        //                                         self.attrs["SALT"],
        //                                         self.attrs["ITER"], 32)
        // print '== Passcode key'
        // print base64.encodestring(anonymize(passcode_key))
        // for classkey in self.classKeys.values():
        //     if not classkey.has_key("WPKY"):
        //         continue
        //     k = classkey["WPKY"]
        //     if classkey["WRAP"] & WRAP_PASSCODE:
        //         k = AESUnwrap(passcode_key, classkey["WPKY"])
        //         if not k:
        //             return False
        //         classkey["KEY"] = k
        // return True
    }

    pub fn unlock_with_passcode(&mut self, passcode: &str) {
        println!("deriving keys...");
        #[cfg(debug_assertions)]
        warn!("key derivation is slow in non-release mode.");
        let mut passcode1 : Vec<u8> = vec![0u8; 32];
        let mut passcode_key : Vec<u8> = vec![0u8; 32];
        
        let dpic = self.dpic.unwrap();
        let iterations = self.iterations.unwrap();
        let double_protection_salt = self.double_protection_salt.as_ref().unwrap();

        debug!("dpic: {}", dpic);
        debug!("dpsl: {:?}", double_protection_salt);
        debug!("iterations: {}", iterations);
        debug!("salt: {:?}", self.salt);
        debug!("deriving keys... (this may take a while)");
        debug!("1. pbkdf2-sha256(it: {}, ps: {})", dpic, passcode);

        // 1. Round of pbkdf2-sha256(passcode)
        pbkdf2::derive(&digest::SHA256, std::num::NonZeroU32::new(dpic as u32).unwrap(), &double_protection_salt.as_slice(), passcode.as_bytes(), passcode1.as_mut_slice());

        // 2. Round of pbkdf2-sha1(pbkdf2-sha256(passcode))
        debug!("done.");
        debug!("2. pbkdf2-sha2(it: {}, ps: {})", iterations, hex::encode(&passcode1));
        pbkdf2::derive(&digest::SHA1, std::num::NonZeroU32::new(iterations as u32).unwrap(), &self.salt.as_slice(), passcode1.as_slice(), passcode_key.as_mut_slice());

        debug!("3. result = {}", hex::encode(&passcode_key));
        debug!("{:?}", passcode_key);

        println!("deriving keys [done]");
        // crypto::pbkdf2::pbkdf2(&mut mac, &self.double_protection_salt.as_slice(), self.dpic, passcode1.as_mut_slice());
        // crypto::pbkdf2::pbkdf2(&mut sha1, &self.salt.as_slice(), self.iterations, passcode_key.as_mut_slice());

        self.unlock_with_key(passcode_key);
    }

    fn init_keybag(root_blocks: Vec<BackupKeyBagBlock>) -> KeyBag {
        let mut version: Option<u32> = None;
        let mut kind: Option<KeybagTypes> = None;
        let mut uuid: Option<Uuid> = None;
        let mut hmck: Option<Vec<u8>> = None;
        let mut salt: Option<Vec<u8>> = None;
        let mut double_protection_salt: Option<Vec<u8>> = None;
        let mut iterations: Option<u32> = None;
        let mut dpwt: Option<u32> = None;
        let mut dpic: Option<u32> = None;
        let mut wrap: Option<u32> = None;

        for block in root_blocks {
            match block.tag {
                KeybagBlockTag::UUID => {
                    uuid = Some(Uuid::parse_str(&hex::encode(&block.data)).unwrap());
                    debug!("found uuid: {:?}", uuid);
                },
                KeybagBlockTag::VERS => {
                    version = Some(u32::from_be_bytes(KeyBag::get_u8_4(block.data.as_slice()).unwrap()));
                    debug!("found version: {:?}", version);
                },
                KeybagBlockTag::TYPE => {
                    kind = Some(KeybagTypes::from(u32::from_be_bytes(KeyBag::get_u8_4(block.data.as_slice()).unwrap())));
                    debug!("found kind: {:?}", kind);
                },
                KeybagBlockTag::ITER => {
                    iterations = Some(u32::from_be_bytes(KeyBag::get_u8_4(block.data.as_slice()).unwrap()));
                    debug!("found iterations: {:?}", iterations);
                },
                KeybagBlockTag::DPWT => {
                    dpwt = Some(u32::from_be_bytes(KeyBag::get_u8_4(block.data.as_slice()).unwrap()));
                    debug!("found dpwt: {:?}", dpwt);
                },
                KeybagBlockTag::DPIC => {
                    dpic = Some(u32::from_be_bytes(KeyBag::get_u8_4(block.data.as_slice()).unwrap()));
                    debug!("found dpic: {:?}", dpic);
                },
                KeybagBlockTag::DPSL => {
                    debug!("found dpsl: {:?}", block.data);
                    double_protection_salt = Some(block.data);
                },
                KeybagBlockTag::SALT => {
                    debug!("found salt: {:?}", block.data);
                    salt = Some(block.data);
                },
                KeybagBlockTag::HMCK => {
                    hmck = Some(block.data);
                    debug!("found hmck: {:?}", hmck);
                },
                KeybagBlockTag::WRAP => {
                    wrap = Some(u32::from_be_bytes(KeyBag::get_u8_4(block.data.as_slice()).unwrap()));
                    debug!("found hmck: {:?}", hmck);
                },
                x => {
                    debug!("cannot handle {:?}", x);
                }
            }
        }


        KeyBag {
            version: version.unwrap(),
            uuid: uuid.unwrap(),
            kind: kind.unwrap(),
            iterations: iterations,
            dpwt: dpwt,
            dpic: dpic,
            double_protection_salt: double_protection_salt,
            salt: salt.unwrap(),
            hmck: hmck.unwrap(),
            wrap: wrap.unwrap(),
            keys: vec![],
            key: None
        }
    }

    fn init_container(blocks: &Vec<BackupKeyBagBlock>) -> KeybagEntry {
        let mut uuid: Option<Uuid> = None;
        let mut class: Option<ProtectionClass> = None;
        let mut key_type: Option<KeyTypes> = None;
        let mut wrap: Option<u32> = None;
        let mut wpky: Option<Vec<u8>> = None;

        for block in blocks {
            match block.tag {
                KeybagBlockTag::UUID => {
                    uuid = Some(Uuid::parse_str(&hex::encode(&block.data)).unwrap());
                    debug!("found uuid: {:?}", uuid);
                },
                KeybagBlockTag::CLAS => {
                    class = Some(ProtectionClass::from(u32::from_be_bytes(KeyBag::get_u8_4(block.data.as_slice()).unwrap())));
                    debug!("found protclass: {:?}", class);
                },
                KeybagBlockTag::KTYP => {
                    key_type = Some(KeyTypes::from(u32::from_be_bytes(KeyBag::get_u8_4(block.data.as_slice()).unwrap())));
                    debug!("found keytype: {:?}", key_type);
                },
                KeybagBlockTag::WRAP => {
                    wrap = Some(u32::from_be_bytes(KeyBag::get_u8_4(block.data.as_slice()).unwrap()));
                    debug!("found wrapper: {:?}", wrap);
                },
                KeybagBlockTag::WPKY => {
                    wpky = Some(block.data.clone());
                    debug!("found wpky: {:?}", block.data);
                },
                _ => {}
            };
        }

        KeybagEntry {
            uuid: uuid.unwrap(),
            class: class.unwrap(),
            key_type: key_type.unwrap(),
            wrap: wrap.unwrap(),
            wpky: wpky.unwrap(),
            key: None
        }
    }

    pub fn init(data: Vec<u8>) -> KeyBag {
        let blocks = KeyBag::parse_tlb_blocks(data).unwrap();
        let root_blocks = KeyBag::find_root_blocks(&blocks);
        let contained_entries = KeyBag::find_contained_blocks(&blocks);

        // debug!("root: {:#?}", root_blocks);
        // debug!("contained: {:#?}", contained_entries);
        let mut keybag = KeyBag::init_keybag(root_blocks);
        keybag.keys = contained_entries.iter().map(|v| KeyBag::init_container(v)).collect::<Vec<KeybagEntry>>();

        keybag
    }

    fn get_u8_4(vec: &[u8]) -> Option<[u8; 4]> {
        if vec.len() < 4 {
            return None
        } else {
            return Some([vec[0], vec[1], vec[2], vec[3]])
        }
    }

    fn parse_tlb_blocks(data: Vec<u8>) -> Option<Vec<BackupKeyBagBlock>> {
        let mut i = 0;
        let mut blocks = vec![];
        
        debug!("parse tlb blocks: {}", data.len());
        while i + 8 < data.len() {
            let tag = match std::str::from_utf8(&data[i..i+4]) {
                Ok(res) => KeybagBlockTag::from(res),
                Err(err) => panic!("Error parsing key type: {}", err)
            };
            let x : [u8; 4] = match KeyBag::get_u8_4(&data[i+4..i+8]) {
                Some(el) => el,
                None => return None
            };
            let length = u32::from_be_bytes(x) as usize;
            let data = Vec::from(&data[i+8..i+8+length]);

            debug!("tag: {:?}, length: {}",  tag, length);
            blocks.push(BackupKeyBagBlock {
                tag, length, data
            });

            i += 8 + length;
        }

        Some(blocks)
    }
}