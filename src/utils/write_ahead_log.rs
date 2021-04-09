use std::{fs::File, path::PathBuf};

pub struct WriteAheadLog{
    path: PathBuf,
    file: File,
    // memory_mapping_file
}