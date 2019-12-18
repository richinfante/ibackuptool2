use std::io::Write;
use crate::lib::*;

pub struct OutFile {
  pub filename: String,
  contents: Vec<u8>
}

impl OutFile {
  pub fn new(name: &str) -> OutFile {
    OutFile {
      filename: name.to_string(),
      contents: vec![]
    }
  }

  pub fn contents(&self) -> &[u8] {
    return self.contents.as_slice()
  }
}

impl std::io::Write for OutFile {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    self.contents.write(buf)
  }

  fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
    self.contents.write_vectored(bufs)
  }

  fn flush(&mut self) -> std::io::Result<()> {
    self.contents.flush()
  }

  fn write_all(&mut self, mut buf: &[u8]) -> std::io::Result<()> {
    self.contents.write_all(&mut buf)
  }

  fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
    self.contents.write_fmt(fmt)
  }

  fn by_ref(&mut self) -> &mut Self where Self: Sized { self }
}

trait JSONOutput {
  fn to_json(&self) -> Vec<OutFile>;
}

trait TextOutput {
  fn to_text(&self, backup: &Backup) -> Vec<OutFile>;
}

