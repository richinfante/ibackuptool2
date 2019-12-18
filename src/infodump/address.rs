use rusqlite::{Connection, NO_PARAMS };
use std::collections::HashMap;
use std::cell::RefCell;
pub type Phone = String;
pub type Email = String;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PropertyType {
  Phone,
  Email,
  Date,
  Website,
  Relationship,
  Ringtone,
  Unknown
}

const PHONE_ID : i64 = 3;
const EMAIL_ID : i64 = 4;
const DATE_ID : i64 = 12;
const WEBSITE_ID : i64 = 22;
const RINGTONE_ID: i64 = 16;
const RELATIONSHIP_ID : i64 = 23;

#[derive(Debug, Clone)]
pub struct Contact {
  pub rowid: u32,
  pub first: Option<String>,
  pub middle: Option<String>,
  pub last: Option<String>,
  pub phones: Vec<(PropertyLabel, String)>,
  pub emails: Vec<(PropertyLabel, String)>
}

#[derive(Debug, Clone)]
pub struct AddressBook {
  pub people: Vec<Contact>
}

impl AddressBook {
  pub fn into_index(&self) -> AddressBookIndexed {
    let mut index : HashMap<String, Vec<Box<Contact>>> = HashMap::new();

    for person in &self.people {
      trace!("indexing: {:?} {:?} id {}", person.first, person.last, person.rowid);
      let cell = Box::new(person.clone());
      for phone in &person.phones {
        let normalized = normalize_phone(&phone.1);
        trace!("indexing: `{}` as `{}`", &phone.1, &normalized);
        if index.contains_key(&normalized) {
          let mut x = index.get_mut(&normalized).unwrap();
          x.push(cell.clone());
        } else {
          index.insert(normalized, vec![cell.clone()]);
        }
      }

      for email in &person.emails {
        trace!("indexing: `{}`", &email.1);
        if index.contains_key(&email.1) {
          let mut x = index.get_mut(&email.1).unwrap();
          x.push(cell.clone());
        } else {
          index.insert(email.1.to_string(), vec![cell.clone()]);
        }
      }
    }


    AddressBookIndexed { index }
  }
}

#[derive(Debug)]
pub struct AddressBookIndexed {
  pub index: std::collections::HashMap<String, Vec<Box<Contact>>>
}

impl AddressBookIndexed {
  pub fn search_via_phone(&self, phone: &str) -> Option<&Vec<Box<Contact>>> {
    let normalized = normalize_phone(phone);

    if self.index.contains_key(&normalized) {
      match self.index.get(&normalized) {
        Some(res) => {
          if res.len() > 0 {
            return Some(res)
          } else {
            return None
          }
        },
        None => { return None }
      }
    } else {
      return None;
    }
  }

  pub fn raw_search(&self, query: &str) -> Option<&Vec<Box<Contact>>> {
    if self.index.contains_key(query) {
      match self.index.get(query) {
        Some(res) => {
          if res.len() > 0 {
            return Some(res)
          } else {
            return None
          }
        },
        None => { return None }
      }
    } else {
      return None;
    }
  }
}

impl PropertyType {
  pub fn from_id(id: i64) -> PropertyType {
    match id {
      PHONE_ID => PropertyType::Phone,
      EMAIL_ID => PropertyType::Email,
      DATE_ID => PropertyType::Date,
      WEBSITE_ID => PropertyType::Website,
      RELATIONSHIP_ID => PropertyType::Relationship,
      RINGTONE_ID => PropertyType::Ringtone,
      _ => PropertyType::Unknown
    }
  }

  pub fn to_id(&self) -> i64 {
    match self {
      PropertyType::Phone => PHONE_ID,
      PropertyType::Email => EMAIL_ID,
      PropertyType::Date => DATE_ID,
      PropertyType::Website => WEBSITE_ID,
      PropertyType::Relationship => RELATIONSHIP_ID,
      PropertyType::Ringtone => RINGTONE_ID,
      PropertyType::Unknown => -1
    }
  }
}

const HOME_ID : i64 = 1;
const OTHER_ID : i64 = 2;
const IPHONE_ID : i64 = 3;
const WORK_ID : i64 = 4;
const MOBILE_ID : i64 = 5;
const MAIN_ID : i64 = 6;
const HOMEFAX_ID : i64 = 10; 
const SCHOOL_ID : i64 = 15;
const WORKFAX_ID : i64 = 16;
const PAGER_ID : i64 = 17;
const ICLOUD_ID : i64 = 18;
const ANNIVERSARY_ID : i64 = 19; 
const HOMEPAGE_ID : i64 = 7;
const MOTHER_ID : i64 = 30;
const FATHER_ID : i64 = 31;
const PARENT_ID : i64 = 32;
const BROTHER_ID : i64 = 33;
const SISTER_ID : i64 = 34;
const SON_ID : i64 = 35;
const DAUGHTER_ID : i64 = 36;
const CHILD_ID : i64 = 37;
const FRIEND_ID : i64 = 38;
const SPOUSE_ID : i64 = 39;
const PARTNER_ID : i64 = 40;
const ASSISTANT_ID : i64 = 41;
const MANAGER_ID : i64 = 42;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PropertyLabel {
  Home,
  Other,
  iPhone,
  Work,
  Mobile,
  Main,
  HomeFax,
  School,
  WorkFax,
  Pager,
  iCloud,
  Anniversary,
  Homepage,
  Mother,
  Father,
  Parent,
  Brother,
  Sister,
  Son,
  Daughter,
  Child,
  Friend,
  Spouse,
  Partner,
  Assistant,
  Manager,
  Unknown
}

impl PropertyLabel {
  pub fn from_id(id: i64) -> PropertyLabel {
    match id {
      HOME_ID => PropertyLabel::Home,
      OTHER_ID => PropertyLabel::Other,
      IPHONE_ID => PropertyLabel::iPhone,
      WORK_ID => PropertyLabel::Work,
      MOBILE_ID => PropertyLabel::Mobile,
      MAIN_ID => PropertyLabel::Main,
      HOMEFAX_ID => PropertyLabel::HomeFax,
      SCHOOL_ID => PropertyLabel::School,
      WORKFAX_ID => PropertyLabel::WorkFax,
      PAGER_ID => PropertyLabel::Pager,
      ICLOUD_ID => PropertyLabel::iCloud,
      ANNIVERSARY_ID => PropertyLabel::Anniversary,
      HOMEPAGE_ID => PropertyLabel::Homepage,
      MOTHER_ID => PropertyLabel::Mother,
      FATHER_ID => PropertyLabel::Father,
      PARENT_ID => PropertyLabel::Parent,
      BROTHER_ID => PropertyLabel::Brother,
      SISTER_ID => PropertyLabel::Sister,
      SON_ID => PropertyLabel::Son,
      DAUGHTER_ID => PropertyLabel::Daughter,
      CHILD_ID => PropertyLabel::Child,
      FRIEND_ID => PropertyLabel::Friend,
      SPOUSE_ID => PropertyLabel::Spouse,
      PARTNER_ID => PropertyLabel::Partner,
      ASSISTANT_ID => PropertyLabel::Assistant,
      MANAGER_ID => PropertyLabel::Manager,
      _ => PropertyLabel::Unknown
    }
  }

  pub fn to_id(&self) -> i64 {
    match self {
      PropertyLabel::Home => HOME_ID,
      PropertyLabel::Other => OTHER_ID,
      PropertyLabel::iPhone => IPHONE_ID,
      PropertyLabel::Work => WORK_ID,
      PropertyLabel::Mobile => MOBILE_ID,
      PropertyLabel::Main => MAIN_ID,
      PropertyLabel::HomeFax => HOMEFAX_ID,
      PropertyLabel::School => SCHOOL_ID,
      PropertyLabel::WorkFax => WORKFAX_ID,
      PropertyLabel::Pager => PAGER_ID,
      PropertyLabel::iCloud => ICLOUD_ID,
      PropertyLabel::Anniversary => ANNIVERSARY_ID,
      PropertyLabel::Homepage => HOMEPAGE_ID,
      PropertyLabel::Mother => MOTHER_ID,
      PropertyLabel::Father => FATHER_ID,
      PropertyLabel::Parent => PARENT_ID,
      PropertyLabel::Brother => BROTHER_ID,
      PropertyLabel::Sister => SISTER_ID,
      PropertyLabel::Son => SON_ID,
      PropertyLabel::Daughter => DAUGHTER_ID,
      PropertyLabel::Child => CHILD_ID,
      PropertyLabel::Friend => FRIEND_ID,
      PropertyLabel::Spouse => SPOUSE_ID,
      PropertyLabel::Partner => PARTNER_ID,
      PropertyLabel::Assistant => ASSISTANT_ID,
      PropertyLabel::Manager => MANAGER_ID,
      PropertyLabel::Unknown => -1
    }
  }
}

/// Normalize a phone number by removing formatting
fn normalize_phone(string: &str) -> String {
  let string1 = string
    .replace(" ", "")
    .replace("+", "")
    .replace("(", "")
    .replace(")", "")
    .replace("-", "");

    if string1.len() == 11 && string1.as_bytes()[0] == '1' as u8 {
      return string1.replacen("1", "", 1);
    }

    return string1
}

/// Normalized compare of phone numbers
pub fn heuristic_phone_same(a: &str, b: &str) -> bool {
  if a == b {
    return true
  }

  return normalize_phone(a) == normalize_phone(b)
}

pub fn get_properties_of_type(kind: PropertyType, inside: &Vec<(PropertyType, PropertyLabel, String)>) -> Vec<(PropertyLabel, String)> {
  return inside.iter().flat_map(|(t, l, v)| {
    if t == &kind {
      return Some((*l, v.clone()))
    }
    return None
  }).collect::<Vec<(PropertyLabel, String)>>();
}

pub fn load_address_book(conn: &Connection) -> Result<AddressBook, Box<dyn std::error::Error>> {
  let mut stmt = conn.prepare("SELECT ROWID, First, Middle, Last from ABPerson").unwrap();

  let contact_iter = stmt.query_map(NO_PARAMS, |row| {
      let rowid : u32 = row.get(0)?;

      Ok(Contact {
        rowid,
        first: row.get(1)?,
        middle: row.get(2)?,
        last: row.get(3)?,
        emails: vec![],
        phones: vec![]
      })
  })?;

  let mut people : Vec<Contact> = vec![];
  for contact in contact_iter {
    match contact {
      Ok(mut contact) => {
        let props = find_listed_properties(conn, contact.rowid)?;
        contact.emails = get_properties_of_type(PropertyType::Email, &props);
        contact.phones = get_properties_of_type(PropertyType::Phone, &props);
        people.push(contact);
      },
      Err(_) => {}
    }
  }

  Ok(AddressBook {
    people
  })
}

pub fn find_listed_properties(conn: &Connection, record_id: u32) -> Result<Vec<(PropertyType, PropertyLabel, String)>, Box<dyn std::error::Error>> {
  let mut stmt = conn.prepare("SELECT ROWID, identifier, property, label, value, guid from ABMultiValue WHERE  record_id = $1")?;
  
  let value_iter = stmt.query_map(vec![record_id], |row| {
      // let rowid : u32 = row.get(0)?;
      // let identifier : i64 = row.get(1)?;
      let property : Option<i64> = row.get(2)?;
      let label : Option<i64> = row.get(3)?;
      let value : String = row.get(4)?;
      // let guid : Option<String> = row.get(5)?;

      Ok((
        PropertyType::from_id(property.unwrap_or(-1)),
        PropertyLabel::from_id(label.unwrap_or(-1)),
        value
      ))
  })?;


  Ok(value_iter.flat_map(|v|v).collect())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_normalize() {
    assert_eq!(normalize_phone("+12345678901"), "2345678901");
    assert_eq!(normalize_phone("234-567-8901"), "2345678901");
    assert_eq!(normalize_phone("(234) 567-8901"), "2345678901");
    assert_eq!(normalize_phone("+1 (234) 567 8901"), "2345678901");
  }

  #[test]
  fn test_heuristic() {
    assert_eq!(normalize_phone("(123) 456 7890"), normalize_phone("+1 (123) 456-7890"));
    assert_eq!(normalize_phone("(123) 456 7890"), normalize_phone("+1 123 456-7890"));
    assert_eq!(normalize_phone("(123) 456 7890"), normalize_phone("+11234567890"));
  }
}