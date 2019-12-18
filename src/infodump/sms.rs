use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS };
use crate::lib::*;
use std::fs::File;
use std::io::{Write, Read, Seek, SeekFrom};
use crate::infodump::*;
use crate::infodump::address::Contact;

use chrono::prelude::DateTime;
use chrono::Utc;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

const IPHONE_2001_EPOCH : i64 = 978307200000; 

#[derive(Debug)]
pub struct Sender {
  rowid: u32,
  id: String,
  country: String,
  service: String
  // uncanonicalized_id
  // person_centric_id
}

impl Sender {
  fn unknown() -> Sender {
    Sender {
      rowid: 0,
      id: String::from("unknown"),
      country: String::from("??"),
      service: String::from("??")
    }
  }

  fn me() -> Sender {
    Sender {
      rowid: 0,
      id: String::from("me"),
      country: String::from("--"),
      service: String::from("--")
    }
  }
}

#[derive(Debug)]
pub struct Message {
  rowid: u32,
  from: Option<Sender>,
  text: Option<String>,
  date: i64,
  date_read: i64,
  date_delivered: i64,
  is_from_me: Option<bool>,
}

#[derive(Debug)]
pub struct Conversation {
  id: u32,
  guid: String,
  chat_identifier: String, // handle_id
  display_name: String,
  group_id: String,
  participants: Vec<Sender>,
  messages: Vec<Message>
}

pub fn find_person(conn: &Connection, handle_id: u32) -> Option<Sender> {
  let mut persion_stmt = conn.prepare("SELECT ROWID, id, country, service from handle WHERE ROWID = $1").unwrap();
  let mut person_iter = persion_stmt.query_map(vec![handle_id], |row| {
      Ok(Sender {
        rowid: row.get(0)?,
        id: row.get(1)?,
        country: row.get(2)?,
        service: row.get(3)?
      })
  }).unwrap();

  match person_iter.next() {
    Some(v) => {
      match v {
        Ok(v) => Some(v),
        Err(_) => None
      }
    },
    None => None
  }
}


pub fn find_message(conn: &Connection, message_id: u32) -> Message {
  let mut persion_stmt = conn.prepare("SELECT ROWID, handle_id, text, date, date_read, date_delivered, is_from_me from message WHERE ROWID = $1").unwrap();
  let mut person_iter = persion_stmt.query_map(vec![message_id], |row| {
      let sender_id: u32 = row.get(1)?;
      let sender = find_person(conn, sender_id);
      Ok(Message {
        rowid: row.get(0)?,
        from: sender,
        text: row.get(2)?,
        date: row.get(3)?,
        date_read: row.get(4)?,
        date_delivered: row.get(5)?,
        is_from_me: row.get(6)?
      })
  }).unwrap();

  person_iter.next().expect("message to be there").unwrap()
}
pub fn find_people(conn: &Connection, chatid: u32) -> Vec<Sender> {
  let mut out: Vec<Sender> = vec![];
  let mut stmt = conn.prepare("SELECT handle_id FROM chat_handle_join WHERE chat_id = $1").unwrap();
  
  let handle_id_iter = stmt.query_map(vec![chatid], |row| {
      let handle_id : u32 = row.get(0)?;
      Ok(handle_id)
  }).unwrap();
  

  for handle_id in handle_id_iter {
    let handle_id = handle_id.unwrap();
    match find_person(&conn, handle_id) {
      Some(p) => { out.push(p) },
      None => {}
    }
  }

  out
}

pub fn find_messages(conn: &Connection, chatid: u32) -> Vec<Message> {
  let mut out: Vec<Message> = vec![];
  let mut stmt = conn.prepare("SELECT chat_id, message_id, message_date FROM chat_message_join WHERE chat_id = $1").unwrap();
  
  let message_id_iter = stmt.query_map(vec![chatid], |row| {
      let message_id : u32 = row.get(1)?;
      Ok(message_id)
  }).unwrap();
  

  for message_id in message_id_iter {
    let message_id = message_id.unwrap();
    out.push(find_message(conn, message_id))
  }

  out
}

pub fn read_chats(conn: &Connection) -> Vec<Conversation> {

  let mut stmt = conn.prepare("SELECT rowid, guid, chat_identifier, display_name, group_id FROM chat").unwrap();
  let chat_iter = stmt.query_map(NO_PARAMS, |row| {
      let chat_id: u32 = row.get(0)?;
      Ok(Conversation {
          id: chat_id,
          guid: row.get(1)?,
          chat_identifier: row.get(2)?,
          display_name: row.get(3)?,
          group_id: row.get(4)?,
          participants: find_people(&conn, chat_id),
          messages: find_messages(&conn, chat_id)
      })
  }).unwrap();

  let convos: Vec<Conversation> = chat_iter.flat_map(|v| v).collect();

  convos
}

pub fn localize_sender_id(index: &crate::infodump::address::AddressBookIndexed, original_sender_name: &str) -> String {
  let mut sender_name = original_sender_name.to_string();
  let mut person : Option<Box<&Contact>> = None;
  
  if sender_name.contains("@") {
    match index.raw_search(&sender_name) {
      Some(val) => {
        trace!("index: got raw: {:?}", val);
        person = Some(Box::new(val[0].as_ref()));
      },
      None => {
        trace!("index: no hit for `{}`", sender_name);
      }
    }
  } else {
    match index.search_via_phone(&sender_name) {
      Some(val) => {
        trace!("index: got phones: {:?}", val);
        person = Some(Box::new(val[0].as_ref()));
      },
      None => {
        trace!("index: no phone hit for `{}`", sender_name);
      }
    }
  }

  match person {
    None => {}
    Some(contact) => {
      let components = vec![contact.first.as_ref(), contact.middle.as_ref(), contact.last.as_ref()];
      let outname = components.iter().flat_map(|v| {
        match v {
          Some(v) => Ok(v.clone()),
          None => Err(())
        }
      }).collect::<Vec<&String>>();

      let mut out_addr = String::new();
      for component in outname {
        out_addr.extend(component.chars());
        out_addr.extend(" ".chars());
      }

      sender_name = format!("{} <{}>", out_addr.trim_end(), sender_name);
    }
  }

  sender_name
}


pub struct SMSReader {
  chats: Vec<Conversation>
}

impl SMSReader {
  pub fn load(backup: &Backup) -> Result<SMSReader, Box<dyn std::error::Error>> {
    let proxy = SqliteProxy::new(backup, "HomeDomain", "Library/SMS/sms.db")?;
    let conn = &proxy.connection;
  
    let chats = read_chats(conn);

    Ok(SMSReader {
      chats
    })
  }
}

impl TextOutputFormat for SMSReader {
  fn to_text(&self, backup: &Backup) -> Result<Vec<OutFile>, Box<dyn std::error::Error>> {
    let addrproxy = SqliteProxy::new(backup, "HomeDomain", "Library/AddressBook/AddressBook.sqlitedb")?;
    let book = crate::infodump::address::load_address_book(&addrproxy.connection)?;
    let index = book.into_index();

    let mut files : Vec<OutFile> = vec![];

    for chat in &self.chats {
      let mut chat_name_display = chat.display_name.clone();
      if chat_name_display.len() == 0 {
        chat_name_display = chat.chat_identifier.clone();
      }

      chat_name_display = localize_sender_id(&index, &chat_name_display);

      let mut outfile = OutFile::new(&format!("{}.txt", chat_name_display));

      for message in &chat.messages {
        let mut sender_name = message.from.as_ref().unwrap_or(&Sender::unknown()).id.to_string();
        if message.is_from_me == Some(true) {
          sender_name = "me".to_string();
        } else {
          sender_name = localize_sender_id(&index, &sender_name);
        }

        let message_epoch : u64 = ((IPHONE_2001_EPOCH + message.date / 1000000) as u64) / 1000;
        trace!("got computed epoch {} to {}", message.date, message_epoch);
        let d = UNIX_EPOCH + Duration::from_secs(message_epoch);
        // Create DateTime from SystemTime
        let datetime = DateTime::<Utc>::from(d);
        // Formats the combined date and time with the specified format string.
        let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S.%f").to_string();
        
        // Output the final message to the terminal.
        writeln!(outfile, "{}: {}: {}: {}", timestamp_str, &chat_name_display, sender_name, message.text.as_ref().unwrap_or(&"(no content)".to_string()))?;
        // println!("{}: {}: {}: {}", timestamp_str, &chat_name_display, sender_name, message.text.unwrap_or("(no content)".to_string()));
      }

      files.push(outfile);
    }

    Ok(files)
  }
}