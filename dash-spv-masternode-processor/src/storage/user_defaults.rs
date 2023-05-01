use plist::{Integer, Value};
use crate::util::file::{preferences_in_read_mode, preferences_in_write_mode};

#[derive(Debug, Default)]
pub struct UserDefaults {

}
// NSDictionary *defaults = @{
// @"useCheckpointMasternodeLists": @YES,
// @"keepHeaders": @NO,
// @"shouldSyncFromHeight": @NO,
// @"smartOutputs": @YES,
// @"syncGovernanceObjectsInterval": @600, // 10 min
// @"syncMasternodeListInterval": @600,    // 10 min
// @"syncFromHeight": @0,
// @"retrievePriceInfo": @YES,
// @"shouldUseCheckpointFile": @YES,
// @"syncType": @(DSSyncType_Default),
// };

impl UserDefaults {

    fn create_defaults() -> plist::Dictionary {
        plist::Dictionary::from_iter([
            ("useCheckpointMasternodeLists", Value::Boolean(true)),
            ("keepHeaders", Value::Boolean(false)),
            ("shouldSyncFromHeight", Value::Boolean(false)),
            ("smartOutputs", Value::Boolean(true)),
            ("syncGovernanceObjectsInterval", Value::Integer(600.into())),
            ("syncMasternodeListInterval", Value::Integer(600.into())),
            ("syncFromHeight", Value::Integer(0.into())),
            ("retrievePriceInfo", Value::Boolean(true)),
            ("shouldUseCheckpointFile", Value::Boolean(true)),
            ("syncType", Value::Integer(0.into())),
        ])
    }

    // fn plist() -> Option<Value> {
    //     plist::from_reader(&preferences()).ok()
    //     // if let Value::Dictionary(mut dict) = Self::plist() {
    //     //
    //     // }
    // }
    //
    // fn set_plist<T>(value: &T) where T: serde::Serialize {
    //     plist::to_writer_xml(preferences(), value)
    //         .expect("Can't save preferences");
    // }
    //
    pub fn set(key: impl AsRef<str>, value: Value) {
        let reader = preferences_in_write_mode();
        if let Ok(Value::Dictionary(mut dict)) = plist::from_reader(&reader) {
            dict.insert(key.as_ref().to_string(), value);
            plist::to_writer_xml(&reader, &dict)
                .expect("Can't save preferences")
        }

    }
    fn get(key: impl AsRef<str>) -> Option<Value> {
        let path = preferences_in_read_mode();
        // let plist: Value = plist::from_reader(&path)
        //     .expect("Can't get preferences plist");
        if let Ok(Value::Dictionary(dict)) = plist::from_reader(&path) {
        // if let Value::Dictionary(dict) = plist {
            dict.get(key.as_ref()).cloned()
        } else {
            None
        }
    }

    pub fn delete(key: impl AsRef<str>) {
        let path = preferences_in_write_mode();
        let plist: Value = plist::from_reader(&path)
            .expect("Can't get preferences plist");
        if let Value::Dictionary(mut dict) = plist {
            dict.remove(key.as_ref());
            plist::to_writer_xml(path, &dict)
                .expect("Can't save preferences");
        }
    }

    pub fn has(key: impl AsRef<str>) -> bool {
        Self::get(key).is_some()
    }

    pub fn string_for_key(key: impl AsRef<str>) -> Option<String> {
        if let Some(Value::String(string)) = Self::get(key) {
            Some(string.clone())
        } else {
            None
        }
        // Self::get(key).ok().and_then(|string| String::from_utf8(string.stdout).ok())
    }

    pub fn set_string(key: impl AsRef<str>, value: String) {
        Self::set(key, Value::String(value))
    }

    // pub fn object_for_key<'a, T>(key: impl AsRef<OsStr>) -> Option<T> where T: TryRead<'a, Endian> {
        // let output = Command::new("defaults")
        //     .arg("read")
        //     .arg(Environment::DOMAIN)
        //     .arg(key)
        //     .output()
        //     .expect("failed to read defaults");
        //
        // let stdout = output.stdout.as_slice();
        // stdout.read_with::<T>(&mut 0, byte::LE).ok()
        //
        // // Self::get(key).ok().and_then(|out| out.stdout.read_with::<T>(&mut 0, endian).ok())
        // match Self::get(key) {
        //     Ok(out) => {
        //         let data = &out.stdout;
        //         data.read_with::<T>(&mut 0, byte::LE).ok()
        //     },
        //     Err(err) => None
        // }
        // let output = Command::new("defaults")
        //     .arg("read")
        //     .arg(DOMAIN)
        //     .arg(key)
        //     .stdout(Stdio::piped())
        //     .spawn()
        //     .expect("failed to spawn command")
        //     .wait_with_output()
        //     .expect("failed to read stdout");
        //
        // let out = output.stdout.as_slice();
        // out.read_with::<T>(&mut 0, endian).ok()
    // }

    // pub fn set_object<'a, T>(key: impl AsRef<OsStr>, value: T) where T: TryWrite<Endian> {
    //     let mut bytes = Vec::<u8>::new();
    //     value.try_write(&mut bytes, byte::LE).expect("Can't write object");
    //     Self::set(key, bytes.to_hex());
    // }
    // impl From<u64> for Integer {

    pub fn uint_for_key<'a, T>(key: impl AsRef<str>) -> Option<T> where T: TryFrom<u64> {
        if let Some(Value::Integer(integer)) = Self::get(key) {
            integer.as_unsigned().and_then(|uint| uint.try_into().ok())
        } else {
            None
        }
    }

    pub fn int_for_key<'a, T>(key: impl AsRef<str>) -> Option<T> where T: TryFrom<i64> {
        if let Some(Value::Integer(integer)) = Self::get(key) {
            integer.as_signed().and_then(|int| int.try_into().ok())
        } else {
            None
        }

    }
    pub fn set_num<T>(key: impl AsRef<str>, value: T) where Integer: From<T> {
        // let mut bytes = Vec::<u8>::new();
        // value.try_write(&mut bytes, byte::BE).expect("Can't write object");
        // Self::set(key, bytes.to_hex());
        Self::set(key, Value::Integer(value.into()));
    }

}
