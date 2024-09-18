use std::io::{Cursor, Read};
// NOTE: this is a direct port of the node.js implemention.
// it may not actually work in all cases.
use binread::{BinRead, BinReaderExt, NullString};
use zip::read;
use byteorder::{ByteOrder, BigEndian};

use super::{Backup, BackupFile, FileInfo};
use sha1::{Sha1, Digest};

#[derive(Debug)]
pub struct ManifestMbdbEntry {
    domain: Option<String>,
    filename: Option<String>,
    linktarget: Option<String>,
    datahash: Option<String>,
    enckey: Option<String>,
    mode: u16,
    inode: u64,
    userid: u32,
    groupid: u32,
    mtime: u32,
    atime: u32,
    ctime: u32,
    filelen: u64,
    flag: u8,
    numprops: u8,
    props: std::collections::HashMap<String, String>
}

impl ManifestMbdbEntry {
    pub fn read_from_cur(cursor: &mut Cursor<Vec<u8>>) -> Result<ManifestMbdbEntry, Box<dyn std::error::Error>> {
        let mut mf = ManifestMbdbEntry {
            domain: read_string_from_cur(cursor, false)?,
            filename: read_string_from_cur(cursor, false)?,
            linktarget: read_string_from_cur( cursor, false)?,
            datahash: read_string_from_cur( cursor, false)?,
            enckey: read_string_from_cur( cursor, false)?,
            mode: u16::read( cursor)?,
            inode: u64::read( cursor)?,
            userid: u32::read( cursor)?,
            groupid: u32::read( cursor)?,
            mtime: u32::read(cursor)?,
            atime: u32::read(cursor)?,
            ctime: u32::read( cursor)?,
            filelen: u64::read( cursor)?,
            flag: u8::read( cursor)?,
            numprops: u8::read( cursor)?,
            props: std::collections::HashMap::new()
        };

        let mut newprops = std::collections::HashMap::new();

        for i in 0..mf.numprops {
            let propname = read_string_from_cur(cursor, true)?.unwrap_or(String::new());
            let propval = read_string_from_cur(cursor, true)?.unwrap_or(String::new());
            trace!("got prop: {}={}", propname, propval);
            newprops.insert(propname, propval);
        }

        mf.props = newprops;

        Ok(mf)
    }
}

pub fn read_string_from_cur(cursor: &mut Cursor<Vec<u8>>, ignore_nullterm: bool) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut sz = [0u8; 2];
    cursor.read_exact(&mut sz)?;

    // NOTE: this length is NOT always right. It's sometimes junk data for reasons I don't understand.
    // Often in these cases, it's still null terminated.
    let len : u16 = BigEndian::read_u16(&sz);
    debug!("len: {} {:#b}", len, len);

    if len == 0xffff {
        trace!("str -> none val (0xFFFF)");
        return Ok(None);
    }

    let mut string = String::new();
    let mut read_ct = 0;

    loop {
        if read_ct >= len {
            break
        }

        read_ct += 1;

        let val = u8::read(cursor)?;

        if val == 0x00 && !ignore_nullterm {
            break
        } else {
            string.push(val as char);
        }
    }

    Ok(Some(string))
}

#[derive(Debug)]
pub struct ManifestMbdb {
    files: Vec<ManifestMbdbEntry>
}

pub fn parse_manifest(backup: &mut Backup) -> Result<(), Box<dyn std::error::Error>> {
    let contents = backup.raw_file_read("Manifest.mbdb").expect("manifest.mbdb to exist");
    
    let contents = parse_manifest_contents(contents);

    for file in &contents.files {

        let domain = {
            let string = String::new();
            let dm = file.domain.as_ref();

            dm.unwrap_or(&string).to_string()
        };
        let relative_filename = {
            let string = String::new();
            let dm = file.filename.as_ref();

            dm.unwrap_or(&string).to_string()
        };

        // create a Sha1 object
        let mut hasher = Sha1::new();
        hasher.update(format!("{}-{}", &domain, &relative_filename).as_bytes());
        let fileid_bytes = hasher.finalize();
        let fileid = format!("{:x}", fileid_bytes);

        backup.files.push(BackupFile {
            domain: domain.clone(),
            fileid,
            relative_filename: relative_filename.clone(),
            flags: file.flag as i64, // TODO: is this right?
            fileinfo: Some(FileInfo {
                last_modified: file.atime as u64,
                last_status_change: 0,
                birth: file.ctime as u64,
                size: file.filelen,
                group_id: file.groupid as u64,
                user_id: file.userid as u64,
                flags: file.flag as u64, // TODO: is this right?
                mode: file.mode as u64,
                inode: file.inode as u64,
                protection_class: crate::lib::ProtectionClass::Unknown,
                wrapped_encryption_key: None,
                wrapped_encryption_class: None,
                encryption_key: None,
                extended_attributes: None
            })
        })
    }


    Ok(())
}

pub fn parse_manifest_contents(contents: Vec<u8>) -> ManifestMbdb {
    let mut reader = Cursor::new(contents);
    let mut val: [u8; 6] = [0,0,0,0,0,0];
    let c1 = reader.read_exact(&mut val);

    if &val != b"mbdb\x05\x00" {
        panic!("missing or invalid header!");
    }

    let mut files = Vec::new();

    loop {
        let entry = match ManifestMbdbEntry::read_from_cur(&mut reader) {
            Ok(v) => v,
            Err(e) => {
                error!("err: {}", e);
                break
            }
        };

        debug!("{:?}", entry);
        files.push(entry);
    }

    ManifestMbdb {
        files
    }
}