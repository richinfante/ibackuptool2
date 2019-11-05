#[macro_use]
extern crate log;
use std::path::Path;

extern crate serde;

extern crate clap;
use clap::{Arg, App, SubCommand};

mod lib;
use lib::*;

const BACKUP_DIRECTORY: &'static str = "/Library/Application Support/MobileSync/Backup/";

fn main() {
    env_logger::init();

    let matches = App::new("ibackuptool2")
                        .version("1.0")
                        .author("Rich <rich@richinfante.com>")
                        .about("iOS Backup Utilities")
                        .arg(Arg::with_name("dir")
                            .short("d")
                            .long("directory")
                            .value_name("DIR")
                            .help("Sets a custom backup origin folder.")
                            .takes_value(true))
                        .subcommand(SubCommand::with_name("ls")
                                    .about("lists backups or files within a backup"))
                        .subcommand(SubCommand::with_name("ls2")
                            .arg(Arg::with_name("backup")
                            .short("b")
                            .long("backup")
                            .value_name("BACKUP")
                            .help("Sets a custom backup name / path. prepended to --directory.")
                            .takes_value(true)))
                        .get_matches();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    // let config = matches.value_of("config").unwrap_or("default.conf");
    // println!("Value for config: {}", config);

    // Calling .unwrap() is safe here because "INPUT" is required (if "INPUT" wasn't
    // required we could have used an 'if let' to conditionally get the value)
    // println!("Using input file: {}", matches.value_of("INPUT").unwrap());

    let home_dir = match dirs::home_dir() {
        Some(res) => match res.to_str() {
            Some(res) => res.to_string(),
            None => panic!("Can't convert homedir to string!"),
        },
        None => panic!("Can't find homedir:"),
    };

    let backup_dir = format!("{}{}", home_dir, BACKUP_DIRECTORY);

    trace!("using src directory: {}", backup_dir);
    let dir = Path::new(&backup_dir);

    if dir.is_dir() {
        trace!("(backup directory exists!)");
    }


    // You can handle information about subcommands by requesting their matches by name
    // (as below), requesting just the name used, or both at the same time
    if let Some(matches) = matches.subcommand_matches("ls") {

        let ls = std::fs::read_dir(dir).unwrap();

        for entry in ls {
            let entry = entry.unwrap();
            if entry.path().is_dir() {
                debug!("reading backup: {:?}", entry.path());
                let path = entry.path();
                match Backup::new(&path) {
                    Ok(mut backup) => {
                        println!(
                            "id={} name={} product={} iOS={} encrypted={:?} dir={:?}",
                            backup.info.target_identifier,
                            &backup
                                .info
                                .device_name
                                .as_ref()
                                .unwrap_or(&"<unnamed device>".to_string()),
                            &backup
                                .info
                                .product_name
                                .as_ref()
                                .unwrap_or(&"<unknown product>".to_string()),
                            backup.info.product_version,
                            &backup.manifest.is_encrypted,
                            &path.file_name().unwrap(),
                        );

                        // if backup.manifest.is_encrypted {
                        //     // Parse the manifest keybag
                        //     backup.parse_keybag().unwrap();
                        //     debug!("trying decrypt of backup keybag");

                        //     // Unlock the keybag with password
                        //     if let Some(ref mut kb) = backup.manifest.keybag.as_mut() {
                        //         let pass = rpassword::read_password_from_tty(Some("Backup Password: "))
                        //             .unwrap();
                        //         kb.unlock_with_passcode(&pass); // TODO:
                        //     }

                        //     // Unlock the manifest key
                        //     backup.manifest.unlock_manifest();

                        //     // Parse the manifest
                        //     backup.parse_manifest().unwrap();

                        //     // now, unwrap all file keys in preparation of doing things; we can do this on a file-by-file basis also.
                        //     backup.unwrap_file_keys().unwrap();
                        // } else {
                        //     backup.parse_manifest().unwrap();
                        // }

                        // info!("loaded {} files from manifest", backup.files.len());
                        // info!(
                        //     "loaded: {} domains from manifest",
                        //     list_domains(&backup).len()
                        // );
                    }
                    Err(err) => info!("failed to load {}: {:?}", err, path),
                };
            }
        }
            
    }

    if let Some(matches) = matches.subcommand_matches("ls2") {
        let path = Path::new(matches.value_of("backup").unwrap());
        if path.is_dir() {
            debug!("reading backup: {:?}", &path);
            match Backup::new(&path) {
                Ok(mut backup) => {
                    println!(
                        "reading backup id={}, name={}, product={}, iOS={}, encrypted={:?}",
                        backup.info.target_identifier,
                        &backup
                            .info
                            .device_name
                            .as_ref()
                            .unwrap_or(&"<unnamed device>".to_string()),
                        &backup
                            .info
                            .product_name
                            .as_ref()
                            .unwrap_or(&"<unknown product>".to_string()),
                        backup.info.product_version,
                        &backup.manifest.is_encrypted
                    );

                    if backup.manifest.is_encrypted {
                        // Parse the manifest keybag
                        backup.parse_keybag().unwrap();
                        debug!("trying decrypt of backup keybag");

                        // Unlock the keybag with password
                        if let Some(ref mut kb) = backup.manifest.keybag.as_mut() {
                            let pass = rpassword::read_password_from_tty(Some("Backup Password: "))
                                .unwrap();
                            kb.unlock_with_passcode(&pass); // TODO:
                        }

                        // Unlock the manifest key
                        backup.manifest.unlock_manifest();

                        // Parse the manifest
                        backup.parse_manifest().unwrap();

                        // now, unwrap all file keys in preparation of doing things; we can do this on a file-by-file basis also.
                        backup.unwrap_file_keys().unwrap();
                    } else {
                        backup.parse_manifest().unwrap();
                    }

                    info!("loaded {} files from manifest", backup.files.len());
                    info!(
                        "loaded: {} domains from manifest",
                        list_domains(&backup).len()
                    );

                    for file in backup.files {
                        println!("{}: {}, {}", file.fileid, file.domain, file.relative_filename);
                    }
                }
                Err(err) => info!("failed to load {}: {:?}", err, path),
            };
        } else {
            error!("path is not a directory: {}", path.display());
        }
            
    }
}

fn list_domains(backup: &Backup) -> Vec<String> {
    let mut domains: std::collections::HashSet<String> = std::collections::HashSet::new();
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
