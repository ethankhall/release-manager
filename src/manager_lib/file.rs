use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

pub(crate) fn read_file_to_string(path: &Path) -> String {
    let mut file = File::open(path).expect("Unable to open the file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read the file");
    return contents;
}

pub(crate) fn read_file_to_bytes(path: &Path) -> Vec<u8> {
    let mut file = File::open(path).expect("Unable to open the file");
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .expect("Unable to read the file");
    return contents;
}

pub(crate) fn write_file(contents: String, path: &Path) {
    let mut file = File::create(path).expect("Unable to open the file");
    file.write_all(contents.as_bytes()).expect("file written");
}
