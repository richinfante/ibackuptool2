
#[derive(Debug, Clone, Copy)]
pub enum BackupError {
  InManifestButNotFound,
  NoFileInfo,
  NoEncryptionKey
}

impl std::fmt::Display for BackupError {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
    write!(formatter, "{:?}", self)
  }
}

// This is important for other errors to wrap this one.
impl std::error::Error for BackupError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
      // Generic error, underlying cause isn't tracked.
      None
  }
}
