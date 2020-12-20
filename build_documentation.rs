use std::{
    fs,
    fs::{read_dir, DirEntry, File},
    io::Write,
    path::Path,
    process::Command,
};

fn build_project(location: impl AsRef<Path>, url_location: &str) {
    let out_directory = std::env::var("OUT_DIR").unwrap();
    let out_directory = Path::new(&out_directory).join(url_location);

    if !out_directory.exists() {
        fs::create_dir(&out_directory).expect("Failed to create output directory");
    }

    let directories = sorted_dir_entries(location, |entry| entry.metadata().unwrap().is_dir());

    let mut table_of_contents = "\
        <div class='panel fade' id='toc'><h2>Table of contents</h2><div class='search js-search seperated' style='margin:0px'><input \
                                 placeholder='Search...' type='text' style='height:1em'></div><ol style='padding-left: 0px'>
    "
    .to_owned();

    for dir in directories {
        let name = dir.file_name();
        let filename = &name.to_str().unwrap()[4..];
        let section_name = filename.replace("_", " ");
        let url_name = filename.to_lowercase().replace("_", "");
        let filename = format!("{}.html", url_name);
        let file = out_directory.join(filename);
        let file = File::create(file).unwrap();

        if section_name != "index" {
            let mut title_name = section_name.to_string();
            if let Some(r) = title_name.get_mut(0..1) {
                r.make_ascii_uppercase()
            }
            table_of_contents.push_str(&format!("<li><a href='/{}/{}'>{}</a><ol>", url_location, url_name, title_name));
        }
        for li in process_directory(&dir, &section_name, url_location) {
            table_of_contents.push_str(&format!("{}", li));
        }

        if section_name != "index" {
            table_of_contents.push_str(&format!("</ol></li>"));
        }

        let mut command = Command::new("pandoc");

        for md in sorted_dir_entries(&dir.path(), |_| true) {
            command.arg(md.path());
        }

        // Prevent pandoc from trying to explicitly set html-table column width on tables with lots
        // of column. Our CSS handles it.
        command.arg("--columns=1000000");

        let output = command.stdout(file).output().unwrap();

        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    table_of_contents.push_str(&format!("</ol>"));
    table_of_contents.push_str(&format!("</div>"));

    let mut file = File::create(out_directory.join("toc.html")).unwrap();

    file.write_all(table_of_contents.as_bytes())
        .expect("Failed to write table of contents");
}

fn main() {
    println!("cargo:rerun-if-changed=./build-documentation.rs");

    build_project(Path::new("./doc"), "documentation");
    build_project(Path::new("./demonlist-guidelines"), "guidelines");
}

fn sorted_dir_entries<F: FnMut(&DirEntry) -> bool, P: AsRef<Path>>(path: P, f: F) -> Vec<DirEntry> {
    let mut entries = read_dir(path).unwrap().filter_map(|r| r.ok()).filter(f).collect::<Vec<_>>();

    for entry in &entries {
        println!("cargo:rerun-if-changed={}", entry.path().to_str().unwrap());
    }

    entries.sort_by_key(|entry| entry.path());

    entries
}

fn find_title<P: AsRef<Path>>(md_file: P) -> (String, String) {
    let cnt = std::fs::read_to_string(md_file).unwrap();

    for line in cnt.split('\n') {
        let trimmed = line.trim();

        if trimmed.starts_with("# ") {
            let headline = String::from(&trimmed[2..]);
            let length = headline.len();

            return match &headline[length - 1..] {
                "}" => {
                    let id_index_start = headline.rfind('=').expect("Malformed header: Missing '='") + 1;
                    let headline_index_end = headline.rfind('{').expect("Malformed header: Missing '{'");

                    (
                        String::from(&headline[id_index_start..length - 1]),
                        String::from(&headline[..headline_index_end]),
                    )
                },
                _ => (headline.replace(' ', "-").to_lowercase(), headline),
            }
        }
    }

    panic!("No headline found in markdown file");
}

fn process_directory(entry: &DirEntry, name: &str, url_location: &str) -> Vec<String> {
    sorted_dir_entries(entry.path(), |_| true)
        .into_iter()
        .map(|entry| find_title(entry.path()))
        .map(|(id, title)| format!("<li><a href = '/{}/{}#{}'>{}</a></li>", url_location, name, id, title))
        .collect()
}
