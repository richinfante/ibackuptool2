A rust port of ibackuptool.

This project is currently under development and is not suitable for normal use yet.

This port will have a smaller scope than the original tool, with it's primary focus being file extraction.
Additional tools may be integrated as separate binaries.

The goals of this project are:
- [x] backup manifest / info parsing
- [ ] backup decryption
  - [x] implement aes key unwrap routines and manifest unlock
  - [x] load manifest into memory as decrypted copy
  - [x] file encryption key unwrap
  - [ ] api for file read access
- [ ] backup file extraction
- [ ] provide rust library for custom use
- [ ] compatibility with old backups via manifest.mbdb




