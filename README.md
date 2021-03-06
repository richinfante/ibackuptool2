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
