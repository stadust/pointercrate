use std::{
    collections::HashMap,
    env,
    ffi::OsStr,
    fs::{read_dir, read_to_string},
    io,
    path::Path,
};

pub fn read_table_of_contents() -> io::Result<String> {
    let env_var = env::var("DOCUMENTATION");
    let doc_files_location = match env_var {
        Ok(ref env_var) => Path::new(env_var),
        Err(_) => Path::new(env!("OUT_DIR")),
    };

    read_to_string(doc_files_location.join("../output"))
}

pub fn read_documentation_topics() -> io::Result<HashMap<String, String>> {
    let env_var = env::var("DOCUMENTATION");
    let doc_files_location = match env_var {
        Ok(ref env_var) => Path::new(env_var),
        Err(_) => Path::new(env!("OUT_DIR")),
    };

    read_dir(doc_files_location)?
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
