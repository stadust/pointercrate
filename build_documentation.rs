use std::{
    fs::{read_dir, DirEntry, File},
    path::Path,
    process::Command,
};

fn main() {
    let doc_directory = Path::new("./doc");
    let out_directory = std::env::var("OUT_DIR").unwrap();
    let out_directory = Path::new(&out_directory);

    let directories = sorted_dir_entries(doc_directory, |entry| entry.metadata().unwrap().is_dir());

    print!("<div class='panel fade' id='toc'>");
    print!("<h2>Table of contents</h2>");
    print!("<div class='search js-search seperated' style='margin:0px'>");
    print!("<input placeholder='Search...' type='text' style='height:1em'>");
    print!("</div>");
    print!("<ol style='padding-left: 0px'>");

    for dir in directories {
        let name = dir.file_name();
        let name = &name.to_str().unwrap()[4..];
        let filename = format!("{}.html", name);
        let file = out_directory.join(filename);
        let file = File::create(file).unwrap();

        if name != "index" {
            let mut title_name = name.to_string();
            if let Some(r) = title_name.get_mut(0..1) {
                r.make_ascii_uppercase()
            }
            print!("<li><a href='/documentation/{}'>{}</a><ol>", name, title_name);
        }
        for li in process_directory(&dir, name) {
            print!("{}", li);
        }

        if name != "index" {
            print!("</ol></li>");
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

    print!("</ol>");
    print!("</div>");
}

fn sorted_dir_entries<F: FnMut(&DirEntry) -> bool, P: AsRef<Path>>(path: P, f: F) -> Vec<DirEntry> {
    let mut entries = read_dir(path).unwrap().filter_map(|r| r.ok()).filter(f).collect::<Vec<_>>();

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

fn process_directory(entry: &DirEntry, name: &str) -> Vec<String> {
    sorted_dir_entries(entry.path(), |_| true)
        .into_iter()
        .map(|entry| find_title(entry.path()))
        .map(|(id, title)| format!("<li><a href = '/documentation/{}#{}'>{}</a></li>", name, id, title))
        .collect()
}
