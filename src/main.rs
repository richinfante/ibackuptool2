#[macro_use]
extern crate log;
use std::path::Path;

extern crate serde;

extern crate clap;
use clap::{App, Arg, SubCommand};

mod lib;
use lib::*;

const BACKUP_DIRECTORY: &'static str = "/Library/Application Support/MobileSync/Backup/";

fn main() {
    env_logger::init();

    let matches = App::new("ibackuptool2")
        .version("1.0")
        .author("Rich <rich@richinfante.com>")
        .about("iOS Backup Utilities")
        .arg(
            Arg::with_name("dir")
                .short("d")
                .long("directory")
                .value_name("DIR")
                .help("Sets a custom backup origin folder.")
                .takes_value(true),
        )
        .subcommand(SubCommand::with_name("ls").about("lists backups or files within a backup"))
        .subcommand(
            SubCommand::with_name("ls-files").arg(
                Arg::with_name("backup")
                    .short("b")
                    .long("backup")
                    .value_name("BACKUP")
                    .help("Sets a custom backup name / path. prepended to --directory.")
                    .takes_value(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("extract")
                .arg(
                    Arg::with_name("backup")
                        .short("b")
                        .long("backup")
                        .value_name("BACKUP")
                        .help("Sets a custom backup name / path. prepended to --directory.")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("dest")
                        .short("o")
                        .long("dest")
                        .value_name("DEST")
                        .help("Extract Destination.")
                        .takes_value(true),
                ),
        )
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

    if let Some(matches) = matches.subcommand_matches("ls-files") {
        let pathloc = matches.value_of("backup").unwrap();
        let path = find_useful_folder(pathloc);
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
                        println!(
                            "{}: {}, {}",
                            file.fileid, file.domain, file.relative_filename
                        );
                    }
                }
                Err(err) => info!("failed to load {}: {:?}", err, path),
            };
        } else {
            error!("path is not a directory: {}", path.display());
        }
    }

    if let Some(matches) = matches.subcommand_matches("extract") {
        let pathloc = matches.value_of("backup").unwrap();
        let extract_dest = Path::new(matches.value_of("dest").unwrap());
        let path = find_useful_folder(pathloc);
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

                    let basepath = Path::new(extract_dest);
                    std::fs::create_dir_all(&basepath);

                    for file in &backup.files {
                        let filepath = basepath
                            .join(Path::new(&file.domain))
                            .join(Path::new(&file.relative_filename));

                        match &backup.read_file(&file) {
                            Ok(res) => {
                                std::fs::create_dir_all(&filepath.parent().unwrap());
                                println!("extract: {}: {} bytes", filepath.display(), res.len());
                                std::fs::write(filepath, res);
                            }
                            Err(err) => {
                                error!("failed to extract: {}: {}", filepath.display(), err);
                            }
                        }
                        // println!("{}: {}, {}", file.fileid, file.domain, file.relative_filename);
                    }
                }
                Err(err) => info!("failed to load {}: {:?}", err, path),
            };
        } else {
            error!("path is not a directory: {}", path.display());
        }
    }
}

fn find_useful_folder(dirname: &str) -> std::path::PathBuf {
    let path = Path::new(dirname);

    println!("useful? {:?}", path.display());
    if path.is_dir() {
        return dirname.into();
    }

    let home_dir = match dirs::home_dir() {
        Some(res) => match res.to_str() {
            Some(res) => res.to_string(),
            None => panic!("Can't convert homedir to string!"),
        },
        None => panic!("Can't find homedir:"),
    };

    let backup_dir = format!("{}{}", home_dir, BACKUP_DIRECTORY);

    let dir = Path::new(&backup_dir);

    return dir.join(Path::new(dirname));
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
