use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Write the website index file.
///
/// If the user has created an index.html in the root, copy this file. Otherwise,
/// generate a new one.
pub fn write_index(project_path: &Path, out_path: &Path) {
    let index_path = project_path.join("index.html");
    let index_out = out_path.join("index.html");
    if index_path.exists() {
        if index_path.is_file() {
            fs::copy(&index_path, &index_out).unwrap();
        }
    } else {
        let index = generate_index();
        let mut file = File::create(&index_out).unwrap();
        file.write_all(index.as_bytes()).unwrap();
    }
}

pub fn generate_index() -> String {
    let mut html = vec![];
    let mut template = include_str!("../../templates/index.html").as_bytes();
    while template.len() > 0 {
        if template.starts_with(b"{TITLE}") {
            let title = b"Title"; // TODO
            html.extend_from_slice(title);
            template = &template[7..];
        } else if template.starts_with(b"{CONTENT}") {
            html.extend_from_slice(b"Content");
            template = &template[9..];
        } else {
            html.push(template[0]);
            template = &template[1..];
        }
    }
    String::from_utf8(html).unwrap()
}
