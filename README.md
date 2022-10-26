A rust port of ibackuptool.

This project is currently under development and is not suitable for normal use yet.

This port will have a smaller scope than the original tool, with it's primary focus being file extraction.
Additional tools may be integrated as separate binaries.

The goals of this project are:

- [x] backup manifest / info parsing
- [x] backup decryption
  - [x] implement aes key unwrap routines and manifest unlock
  - [x] load manifest into memory as decrypted copy
  - [x] file encryption key unwrap
  - [x] api for file read access
- [x] backup file extraction
- [ ] provide rust library for custom use
- [ ] compatibility with old backups via manifest.mbdb

## Usage

### 1. List Available Backups

```bash
$ ibackuptool2 ls
id=6159067247acb912ceb1fbc0f54ae7d2dd693d87 name=iPad product=iPad Air 2 iOS=13.1 encrypted=true dir="6159067247acb912ceb1fbc0f54ae7d2dd693d87"
```

### 2a. List Files in Backup

```bash
$ ibackuptool2 ls-files -b "6159067247acb912ceb1fbc0f54ae7d2dd693d87"
# output format: id, domain, relativePath
735f4f65879e10473dae4050ceee99fbb69de281: CameraRollDomain, Media
# .. [truncated]
```

### 2b. Extract Files in Backup

```bash
$ ibackuptool2 extract -b "6159067247acb912ceb1fbc0f54ae7d2dd693d87" -o "./output"
# (status output for each file that's extracted)
```

## Loading backups from non-default path

Instead of a backup ID, you can also pass a file/folder to the `-b` option, which will attempt to load the backup from that path.

For example:

`ibackuptool2 ls-files -b ~/Documents/zipped_backup.zip`
`ibackuptool2 ls-files -b ~/Documents/zipped_backup_folder`

If you have a folder of backups stored in a non-default location, you can pass the `-d` option to find them:


## Credits

Much of this is based off of my original implementation which was written in Javascript: https://github.com/richinfante/iphonebackuptools

For encryption code, I referenced the python implementation used in this post on stackoverflow: https://stackoverflow.com/questions/1498342/how-to-decrypt-an-encrypted-apple-itunes-iphone-backup
