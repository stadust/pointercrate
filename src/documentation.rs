use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{read_dir, read_to_string},
    io,
    path::Path,
};

pub fn read_table_of_contents(documentation_project: &str) -> io::Result<String> {
    read_to_string(Path::new(documentation_project).join("toc.html"))
}

pub fn read_topics(documentation_project: &str) -> io::Result<HashMap<String, String>> {
    read_dir(Path::new(documentation_project))?
        .map(|result| result.map(|entry| entry.path()))
        .filter(|result| {
            match result {
                Err(_) => true,
                Ok(path) => path.extension().and_then(OsStr::to_str) == Some("html"),
            }
        })
        .map(|result| result.and_then(|path| Ok((path.file_stem().and_then(OsStr::to_str).unwrap().to_string(), read_to_string(path)?))))
        .collect()
}
