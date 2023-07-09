use std::fs::File;
use std::{fs, fs::OpenOptions, io, io::{Read, Write}};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use serde::{de, Serialize};
use crate::environment::Environment;


pub fn create_file(name: &str) -> io::Result<File> { // "processor.log"
    let cache_path = match dirs_next::cache_dir() {
        Some(path) => path,
        None => panic!("Failed to find the cache directory"),
    };
    let dir = cache_path.join("Logs");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .expect("Failed to create 'Logs' directory");
    }
    let file_path = dir.join(name);
    File::create(file_path)
}

pub fn save_json_file<T: Serialize>(name: &str, value: &T) -> io::Result<()> {
    create_file(name)
        .and_then(|mut out| out.write(serde_json::to_string_pretty(value).unwrap().as_ref()).map(|_| ()))
}

pub fn save_java_class(name: &str, contents: &[u8]) -> io::Result<()> {
    create_file(name)
        .and_then(|mut out| out.write(contents).map(|_| ()))
}

/////////////////
fn resource_filepath(name: impl AsRef<str>) -> impl AsRef<str> {
    println!("manifest dir: {}", Environment::cargo_manifest_dir());
    println!("current dir: {}", Environment::cargo_current_dir().to_str().unwrap());
    format!("{}/resources/{}", Environment::cargo_manifest_dir(), name.as_ref())
}

fn file_filepath(name: impl AsRef<str>) -> impl AsRef<str> {
    format!("{}/files/{}", Environment::cargo_manifest_dir(), name.as_ref())
}

fn preferences_filepath() -> impl AsRef<str> {
    format!("{}/Library/Preferences/{}.plist", Environment::cargo_home_dir(), Environment::DOMAIN)
}
/////////////////



fn file_with_metadata(filename: impl AsRef<str>) -> io::Result<(fs::File, fs::Metadata)> {

    fs::File::open(filename.as_ref())
        .and_then(|f| fs::metadata(filename.as_ref())
            .and_then(|meta| Ok((f, meta))))
}

fn file_contents(filename: impl AsRef<str>) -> io::Result<Vec<u8>> {
    file_with_metadata(filename).and_then(|(mut file, meta)| {
        let mut buffer = vec![0; meta.len() as usize];
        file.read_exact(&mut buffer).map(|()| buffer)
    })
}

fn file_contents_force(filename: impl AsRef<str>) -> Vec<u8> {
    println!("file_contents_force: {}", filename.as_ref());
    file_contents(filename).unwrap()
}

pub fn message_from_file(name: String) -> Vec<u8> {
    file_contents_force(&file_filepath(name))
}

pub fn get_resource(name: &str) -> Vec<u8> {
    file_contents_force(&resource_filepath(name))
}

pub fn get_plist<T: de::DeserializeOwned>(contents: &str) -> Result<T, plist::Error> {
    plist::from_bytes(contents.as_bytes())
}

pub fn preferences_in_read_mode() -> fs::File {
    let filepath = preferences_filepath();
    if let Ok(metadata) = fs::metadata(filepath.as_ref()) {
        // check if the file is a file and has read permission
        if metadata.is_file() && /*metadata.permissions().readonly()*/ metadata.permissions().mode() & 0o222 != 0 {
            // do something with the file
            OpenOptions::new().read(true).open(filepath.as_ref()).unwrap()
        } else {
            eprintln!("You don't have permission to access the file");
            let permissions = metadata.permissions();
            let mut new_permissions = permissions.clone();
            new_permissions.set_mode(0o644); // set the file permissions to read and write for the current user
            fs::set_permissions(filepath.as_ref(), new_permissions)
                .expect("Can't set permissions for preferences file");

            OpenOptions::new().read(true).create(true).open(filepath.as_ref()).unwrap()
        }
    } else {
        eprintln!("File not found");
        let file = OpenOptions::new().write(true).create(true).truncate(true).mode(0o644).open(filepath.as_ref())
            .expect("Can't create new preferences file");
        // let file = fs::File::create(filepath.as_ref()).expect("can't create");
        // let mut perms = file.metadata()?.permissions();
        // perms.set_mode(0o644);
        // file.set_permissions(perms).expect("Can't modify permissions");
        file
        // OpenOptions::new().read(true).create(true).open(filepath.as_ref()).unwrap()
    }

    // OpenOptions::new().read(true).create(true).open(preferences_filepath().as_ref()).unwrap()
}

pub fn preferences_in_write_mode() -> fs::File {
    OpenOptions::new().read(true).write(true).create(true).open(preferences_filepath().as_ref()).unwrap()
}

// pub fn preferences() -> fs::File {
//     //
//     //
//     //
//     // let file = std::fs::OpenOptions::new().read(true).open(path)?;
//     // let metadata = file.metadata()?;
//     // let mut buffer = vec![0; metadata.len() as usize];
//     // let mut reader = BufReader::new(file);
//     // reader.read_exact(&mut buffer)?;
//     // Ok(buffer)
//     //
//
//     OpenOptions::new()
//         .read(true)
//         .write(true)
//         .create(true)
//         // .mode()
//         .open(preferences_filepath().as_ref())
//         .expect("Failed to open file")
// }

