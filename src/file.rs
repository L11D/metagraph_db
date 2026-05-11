use std::{fs::{File, OpenOptions}, io::{Seek, SeekFrom, Write}, path::Path};

pub fn append(path: impl AsRef<Path>) -> File {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap()
}

pub fn write(path: impl AsRef<Path>) -> File {
    OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)
        .unwrap()
}

pub fn read(path: impl AsRef<Path>) -> File {
    OpenOptions::new()
        .read(true)
        .open(path)
        .unwrap()
}



pub fn write_all<T: Write>(f: &mut T, data: &[u8]) {
    f.write_all(data).unwrap();
}

pub fn get_end(f: &File) -> u32 {
    f.metadata().unwrap().len() as u32
}

pub fn seek_end(f: &mut File, offset: i64) {
    f.seek(SeekFrom::End(offset)).unwrap();
}

pub fn seek_start(f: &mut File, offset: u32) {
    f.seek(SeekFrom::Start(offset as u64)).unwrap();
}