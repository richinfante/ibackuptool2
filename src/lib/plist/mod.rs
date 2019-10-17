use plist::Value;

pub fn decode_nskeyedarchiver(value: plist::Value) -> plist::Value {
    let mut rot = plist::Dictionary::new();

    // First, ensure the top-level is a dictionary.
    if let Value::Dictionary(root) = value {
        // Next, ensure that this item is actually created by NSKeyedArchiver
        if let Some(Value::String(string)) = root.get("$archiver") {
            if string != "NSKeyedArchiver" {
                panic!("not built by NSKeyedArchiver - bailing.");
            }
        }

        // Resolve the top-level id as a usize index
        let top_uid = match root.get("$top") {
            Some(Value::Dictionary(dict)) => match dict.get("root") {
                Some(Value::Uid(val)) => Some(val.get() as usize),
                _ => None,
            },
            _ => None,
        };

        // Try to get the object container
        let objects = match root.get("$objects") {
            Some(Value::Array(objs)) => objs,
            _ => panic!("no objects!"),
        };

        // If we have a root uuid try to get it
        if let Some(root_uid) = top_uid {
            let root = &objects[root_uid];
            // read referenced object as dict
            if let Some(dict) = root.as_dictionary() {
                // for each key, unwrap it into it's referenced uid object or self.
                for (k, v) in dict.iter() {
                    match v {
                        Value::Uid(uid) => {
                            let uid = uid.get() as usize;
                            let referenced = &objects[uid];
                            rot.insert(k.to_string(), referenced.clone());
                        }
                        _ => {
                            rot.insert(k.to_string(), v.clone());
                        }
                    }
                }
            }
        } else {
            panic!("no root uid specified!");
        }
    } else {
        panic!("malformed keyedarchiver - root is not dict.")
    }

    Value::Dictionary(rot)
}
