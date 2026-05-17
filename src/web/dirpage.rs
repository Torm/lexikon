use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
use crate::document::{DirCrumb, Documents};

// TODO: Make an actual tree of Dir crumbs so we don't have to iterate over all documents.

// pub fn write_doc_dir_pages(doc_out_dir: &Path, documents: &Documents) {
//     for document in documents {
//         let dir_crumbs
//         let mut document_path = doc_out_dir.to_path_buf();
//         for node in &document.dir_crumbs {
//             document_path.push(&node.dir_name);
//             if !fs::exists(&document_path).unwrap() { // Create the directory if it does not exist.
//                 fs::create_dir(&document_path).unwrap();
//             }
//         }
//         document_path.push("index.html");
//         if !fs::exists(&document_path).unwrap() {
//             let dir_index = generate_dir_page(documents)?;
//             let mut file = File::create(&document_path).unwrap();
//             file.write_all(dir_index.as_bytes()).unwrap();
//         }
//     }
// }

// fn generate_dir_page(doc_out_dir: &Path, documents: &Documents, dir_crumb: &Rc<DirCrumb>) -> String {
//     let subdocuments = vec![];
//     let subdirs = vec![];
//     for document in documents {
//         let mut document_path = doc_out_dir.to_path_buf();
//         for node in &document.dir_crumbs {
//             document_path.push(&node.dir_name);
//         }
//         if document_path. {
//
//         }
//
//     }
// }
