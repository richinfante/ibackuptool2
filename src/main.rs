
#[macro_use]
extern crate log;
use std::path::Path;

extern crate serde;

mod lib;
use lib::*;

const BACKUP_DIRECTORY : &'static str = "/Library/Application Support/MobileSync/Backup/";

fn main() {
    env_logger::init();

    let home_dir = match dirs::home_dir() {
        Some(res) => match res.to_str() {
            Some(res) => res.to_string(),
            None => panic!("Can't convert homedir to string!")
        },
        None => panic!("Can't find homedir:")
    };

    let backup_dir = format!("{}{}",
        home_dir,
        BACKUP_DIRECTORY
    );
    
    println!("using directory: {}", backup_dir);
    let dir = Path::new(&backup_dir);

    if dir.is_dir() {
        println!("exists!");
    }

    let ls = std::fs::read_dir(dir).unwrap();

    for entry in ls {
        let entry = entry.unwrap();
        if entry.path().is_dir() {
            println!("{:?}", entry.path());
            let path = entry.path();
            match Backup::new(&path) {
                Ok(mut backup) => {
                    println!("reading backup id={}, name={}, product={}, iOS={}, encrypted={:?}", 
                        backup.info.target_identifier,
                        &backup.info.device_name.as_ref().unwrap_or(&"<unnamed device>".to_string()),
                        &backup.info.product_name.as_ref().unwrap_or(&"<unknown product>".to_string()),
                        backup.info.product_version,
                        &backup.manifest.is_encrypted
                    );

                    if backup.manifest.is_encrypted {
                        // panic!("Decryption of backups is not yet supported.");
                        // println!("{:#?}", backup);
                        // let mut kb = KeyBag::init(backup.manifest.backup_key_bag);
                        // println!("{:#?}", kb);
                        backup.parse_keybag().unwrap();
                        debug!("trying decrypt of backup keybag");
                        if let Some(ref mut kb) = backup.manifest.keybag.as_mut() {
                            let pass = rpassword::read_password_from_tty(Some("Backup Password: ")).unwrap();
                            kb.unlock_with_passcode(&pass); // TODO:
                        }
                        backup.manifest.unlock_manifest();
                        backup.parse_manifest().unwrap();
                    } else {
                        backup.parse_manifest().unwrap();
                    }
                    

                    // println!("{:#?}", kb);
                    println!("loaded {} files from manifest", backup.files.len());
                    println!("loaded: {} domains from manifest", list_domains(&backup).len());
                },
                Err(err) => {
                    println!("failed to load {}: {:?}", err, path)
                }
            };
        }
    }
}

fn list_domains(backup: &Backup) -> Vec<String> {
    let mut domains : std::collections::HashSet<String> = std::collections::HashSet::new();
    for file in &backup.files {
        if !domains.contains(&file.domain) {
            domains.insert(file.domain.to_string());
        }
    }

    return domains.into_iter().collect();
}



// fn print_info() {
//     let value = Value::from_file("tests/data/xml.plist").unwrap();

// }