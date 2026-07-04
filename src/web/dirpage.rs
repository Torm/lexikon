use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use crate::article::Articles;
use crate::compile::project::ResolutionPaths;
use crate::dir::Dir;
use crate::document::{Documents};
use crate::style::Styles;
use crate::web::document::{write_document, write_documents};

pub fn write_dir_indexes(
    styles: &Styles, resolve_paths: &ResolutionPaths, articles: &Articles,
    parent_path: &Path, web_parent_path: &Path,
    documents: &Documents, tree: &Rc<Dir>
) -> Result<(), String> {

    let file_name = tree.file_name.as_os_str();

    let dir_path = parent_path.join(file_name);
    let web_dir_path = web_parent_path.join(file_name);

    write_dir_index(dir_path.as_path(), web_dir_path.as_path(), &tree)?;

    for subtree in &tree.subdirs {
        write_dir_indexes(styles, resolve_paths, articles, dir_path.as_path(), web_dir_path.as_path(), documents, subtree)?
    }

    for document in &tree.subdocs {
        write_document(styles, resolve_paths, articles, dir_path.as_path(), document)?;
    }

    //let doc_out_dir = dir_path.join(tree.file_name.as_os_str());

    for document in documents {

        //let mut document_path = doc_out_dir.to_path_buf();
        //for node in &document.dir_crumbs {
        //    document_path.push(&node.dir_name);
        //    if !fs::exists(&document_path).unwrap() { // Create the directory if it does not exist.
        //        fs::create_dir(&document_path).unwrap();
        //    }
        //}
        // TODO: Follow tree to create dirs.


        //let dirtrail = document.dirtrail();
        //for dir in dirtrail {
        //    path.push('/');
        //    path.push_str(&dir.file_name.to_str().unwrap());
        //    let crumb = dir.name.as_str();
        //    let link = format!(r#"<a href="{}">{}</a>"#, &path, crumb);
        //    html.extend_from_slice(link.as_bytes());
        //}

        //document_path.push("index.html");
        //if !fs::exists(&document_path).unwrap() {
        //    let dir_index = generate_dir_page(doc_out_dir, documents);
        //    let mut file = File::create(&document_path).unwrap();
        //    file.write_all(dir_index.as_bytes()).unwrap();
        //}
    }

    Ok(())
}

fn write_dir_index(path: &Path, web_path: &Path, dir: &Rc<Dir>) -> Result<(), String> {

    let mut html = String::new();

    if !fs::exists(&path).unwrap() { // Create the directory if it does not exist.
        fs::create_dir(&path).unwrap();
    }

    let name = dir.name.as_str();
    html.push_str(format!("<h1><span>{name}</span></h1>").as_str());
    html.push_str("<ul>");
    for document in &dir.subdocs {
        let file_name = document.file_name.as_os_str().to_str().unwrap();
        let file_name = if file_name.ends_with(".doc.khi") {
            file_name.trim_end_matches(".doc.khi")
        } else if file_name.ends_with(".document.khi") {
            file_name.trim_end_matches(".document.khi")
        } else {
            unreachable!();
        };
        let file_name = format!("{}.html", file_name);
        let web_file_path = web_path.join(file_name);
        html.push_str(format!(r#"<li><a href="{}">{}</a></li>"#, web_file_path.to_str().unwrap(), document.title).as_str());
    }
    html.push_str("</ul>");

    for subdir in &dir.subdirs {
        let path = path.join(subdir.file_name.as_os_str());
        let web_path = web_path.join(subdir.file_name.as_os_str());
        write_dir_index_inner(&mut html, path.as_path(), web_path.as_path(), subdir, 2)?;
    }

    let html = generate_dir_page(dir.clone(), html.as_str())?;

    let index_path = path.join("index.html");

    eprintln!("{}", index_path.display());
    //if !fs::exists(&dir_path_index).unwrap() {
    //    let mut file = File::create(&dir_path_index).unwrap();
    //    file.write_all(html.as_bytes()).unwrap();
    //}
    let mut file = File::create(&index_path).unwrap();
    file.write_all(html.as_bytes()).unwrap();

    Ok(())
}

fn write_dir_index_inner(html: &mut String, path: &Path, web_path: &Path, subtree: &Rc<Dir>, level: usize) -> Result<(), String> {
    if level > 6 {
        return Err(format!("Nesting too deep."));
    }

    if !fs::exists(&path).unwrap() { // Create the directory if it does not exist.
        fs::create_dir(&path).unwrap();
    }

    let name = subtree.name.as_str();
    html.push_str(format!("<h{level}><span>{name}</span></h{level}>").as_str());
    html.push_str("<ul>");
    for document in &subtree.subdocs {
        let file_name = document.file_name.as_os_str().to_str().unwrap();
        let file_name = if file_name.ends_with(".doc.khi") {
            file_name.trim_end_matches(".doc.khi")
        } else if file_name.ends_with(".document.khi") {
            file_name.trim_end_matches(".document.khi")
        } else {
            unreachable!();
        };
        let file_name = format!("{}.html", file_name);
        let web_file_path = web_path.join(file_name);
        html.push_str(format!(r#"<li><a href="{}">{}</a></li>"#, web_file_path.to_str().unwrap(), document.title).as_str());
    }
    html.push_str("</ul>");

    for subdir in &subtree.subdirs {
        let path = path.join(subdir.file_name.as_os_str());
        let web_path = web_path.join(subdir.file_name.as_os_str());
        write_dir_index_inner(html, path.as_path(), web_path.as_path(), subdir, level + 1)?;
    }

    Ok(())
}

fn generate_dir_page(dir: Rc<Dir>, content: &str) -> Result<String, String> {
    let mut html = vec![];
    let mut template = include_str!("../../templates/dirpage.html").as_bytes();
    while template.len() > 0 {
        if template.starts_with(b"{TITLE}") {
            let title = dir.name.as_str();
            html.extend_from_slice(title.as_bytes());
            template = &template[7..];
        } else if template.starts_with(b"{NAV}") { // TODO: Replace w Nav path
            //let mut path = String::from("/documents");
            let mut path = String::from("");
            let mut dirtrail = dir.dirtrail(); // TODO Do not include start node in dir trail.
            dirtrail.pop();
            for dir in dirtrail {
                path.push('/');
                path.push_str(&dir.file_name.to_str().unwrap());
                let crumb = dir.name.as_str();
                let link = format!(r#"<a href="{}">{}</a>"#, &path, crumb);
                html.extend_from_slice(link.as_bytes());
            }
            template = &template[5..];
        } else if template.starts_with(b"{CONTENT}") {
            html.extend_from_slice(content.as_bytes());
            template = &template[9..];
        } else {
            html.push(template[0]);
            template = &template[1..];
        }
    }
    Ok(String::from_utf8(html).unwrap())
}
