#[macro_use]
extern crate log;
use std::path::Path;

extern crate serde;

extern crate clap;
use clap::{App, Arg, SubCommand};

mod lib;
use lib::*;
use std::io::Write;
mod infodump;
use infodump::outputformat::*;

const BACKUP_DIRECTORY: &'static str = "/Library/Application Support/MobileSync/Backup/";

fn main() {
    use env_logger::{Builder, Target};

    let mut builder = Builder::from_default_env();
    builder.target(Target::Stderr);

    builder.init();

    let matches = App::new("ibackuptool2")
        .version("1.0")
        .author("Rich <rich@richinfante.com>")
        .about("iOS Backup Utilities")
        .arg(
            Arg::with_name("DIR")
                .short("d")
                .long("directory")
                .value_name("DIR")
                .help("Sets a custom backup origin folder.")
                .takes_value(true),
        )
        .subcommand(SubCommand::with_name("ls").about("lists backups or files within a backup"))
        .subcommand(
            SubCommand::with_name("ls-files").arg(
                Arg::with_name("BACKUP")
                    .short("b")
                    .long("backup")
                    .value_name("BACKUP")
                    .help("Sets a custom backup name / path. prepended to --directory.")
                    .takes_value(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("infodump")
                .arg(
                    Arg::with_name("BACKUP")
                        .short("b")
                        .long("backup")
                        .value_name("BACKUP")
                        .help("Sets a custom backup name / path. prepended to --directory.")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("FORMAT")
                        .short("f")
                        .long("format")
                        .value_name("FORMAT")
                        .possible_values(&["json", "csv", "txt"])
                        .help("Raw infodump capabilities.")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("DEST")
                        .short("o")
                        .long("dest")
                        .value_name("DEST")
                        .help("Extract Destination.")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("find")
                .arg(
                    Arg::with_name("BACKUP")
                        .short("b")
                        .long("backup")
                        .value_name("BACKUP")
                        .help("Sets a custom backup name / path. prepended to --directory.")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("PATH")
                        .long("path")
                        .value_name("PATH")
                        .help("The relativeFilename to find")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("DOMAIN")
                        .long("domain")
                        .value_name("DOMAIN")
                        .help("The domain to find")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("extract")
                .arg(
                    Arg::with_name("BACKUP")
                        .short("b")
                        .long("backup")
                        .value_name("BACKUP")
                        .help("Sets a custom backup name / path. prepended to --directory.")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("DEST")
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

    let mut backup_dir = format!("{}{}", home_dir, BACKUP_DIRECTORY);

    match matches.value_of("DIR") {
        Some(dir) => backup_dir = dir.to_string(),
        _ => {
            trace!("using default backup dir, --directory not specified.")
        }
    }

    trace!("using src directory: {}", backup_dir);
    let dir = Path::new(&backup_dir);

    if dir.is_dir() {
        trace!("(backup directory exists!)");
    }

    // You can handle information about subcommands by requesting their matches by name
    // (as below), requesting just the name used, or both at the same time
    if let Some(_matches) = matches.subcommand_matches("ls") {
        let ls = std::fs::read_dir(dir).unwrap();

        for entry in ls {
            let entry = entry.unwrap();
            if entry.path().is_dir() {
                debug!("reading backup: {:?}", entry.path());
                let path = entry.path();
                match Backup::new(&path) {
                    Ok(backup) => {
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
        let pathloc = matches
            .value_of("BACKUP")
            .expect("expect a backup be passed as an argument");
        let path = find_useful_folder(pathloc);
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
                        let pass =
                            rpassword::read_password_from_tty(Some("Backup Password: ")).unwrap();
                        kb.unlock_with_passcode(&pass); // TODO:
                    }

                    // Unlock the manifest key
                    backup.manifest.unlock_manifest();

                    // Parse the manifest
                    backup.parse_manifest().expect("manifest to be parsed");

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
    }

    if let Some(matches) = matches.subcommand_matches("find") {
        let pathloc = matches.value_of("BACKUP").unwrap();
        let path = find_useful_folder(pathloc);

        debug!("reading backup: {:?}", &path);
        match Backup::new(&path) {
            Ok(mut backup) => {
                debug!(
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
                        let pass =
                            rpassword::read_password_from_tty(Some("Backup Password: ")).unwrap();
                        kb.unlock_with_passcode(&pass); // TODO:
                    }

                    // Unlock the manifest key
                    backup.manifest.unlock_manifest();

                    // Parse the manifest
                    backup.parse_manifest().unwrap();
                } else {
                    backup.parse_manifest().unwrap();
                }

                let mut file = backup
                    .find_path(
                        matches.value_of("DOMAIN").expect("--domain to be provided"),
                        matches.value_of("PATH").expect("--path to be provided"),
                    )
                    .expect("File to exist");

                if backup.manifest.is_encrypted {
                    file.unwrap_file_key(&backup);
                }

                match backup.read_file(&file) {
                    Ok(contents) => match std::io::stdout().write(&contents) {
                        Ok(_) => {}
                        Err(err) => error!("error: {}", err),
                    },
                    Err(err) => error!("error: {}", err),
                }
            }
            Err(err) => info!("failed to load {}: {:?}", err, path),
        };
    }

    if let Some(matches) = matches.subcommand_matches("infodump") {
        let pathloc = matches.value_of("BACKUP").unwrap();
        let dest = Path::new(matches.value_of("DEST").unwrap());
        let path = find_useful_folder(pathloc);
        debug!("reading backup: {:?}", &path);
        match Backup::new(&path) {
            Ok(mut backup) => {
                debug!(
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
                        let pass =
                            rpassword::read_password_from_tty(Some("Backup Password: ")).unwrap();
                        kb.unlock_with_passcode(&pass); // TODO:
                    }

                    // Unlock the manifest key
                    backup.manifest.unlock_manifest();

                    // Parse the manifest
                    backup.parse_manifest().unwrap();
                } else {
                    backup.parse_manifest().unwrap();
                }

                let smsr = infodump::SMSReader::load(&backup).unwrap();
                let files = smsr.to_text(&backup).unwrap();

                for file in files {
                    std::fs::write(
                        dest.join(Path::new("sms/")).join(Path::new(&file.filename)),
                        file.contents(),
                    )
                    .unwrap();
                }
            }
            Err(err) => info!("failed to load {}: {:?}", err, path),
        };
    }

    if let Some(matches) = matches.subcommand_matches("extract") {
        let pathloc = matches.value_of("BACKUP").unwrap();
        let extract_dest = Path::new(matches.value_of("DEST").unwrap());
        let path = find_useful_folder(pathloc);
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
                        let pass =
                            rpassword::read_password_from_tty(Some("Backup Password: ")).unwrap();
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
                std::fs::create_dir_all(&basepath).expect("directory creation to succeed");

                for file in &backup.files {
                    let filepath = basepath
                        .join(Path::new(&file.domain))
                        .join(Path::new(&file.relative_filename));

                    match &backup.read_file(&file) {
                        Ok(res) => {
                            std::fs::create_dir_all(
                                &filepath.parent().expect("expect path to have a parent"),
                            )
                            .expect("directory creation to succeed");
                            println!("extract: {}: {} bytes", filepath.display(), res.len());
                            std::fs::write(filepath, res)
                                .expect("to be able to write file contents");
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
    }
}

fn find_useful_folder(dirname: &str) -> std::path::PathBuf {
    let path = Path::new(dirname);

    debug!("check useful? {:?}", path.display());
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
