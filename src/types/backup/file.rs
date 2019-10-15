#[derive(Debug)]
pub struct BackupFile {
  /// This corresponds to the hash of the file id on disk.
  /// fileid equals sha1(format!("{}-{}", domain, relative_filename))
  pub fileid: String,

  /// This corresponds to the domain the file is contained in. 
  /// example: "MediaDomain" or "CameraRollDomain"
  pub domain: String,

  /// The file path, relative to the domain
  pub relative_filename: String
}