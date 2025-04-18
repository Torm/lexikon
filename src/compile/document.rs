use std::rc::Rc;
use crate::compile::class::Article;

/// A document that also describes its location in the file system.
pub struct FsDocument<'a> {
    pub(crate) key: String,
    pub(crate) dir_path: Vec<Rc<DirNode>>,
    pub(crate) file_name: String,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    pub(crate) preamble: Option<String>,
    pub(crate) resolution_paths: Vec<String>,
    pub(crate) structure: Vec<DocumentElement<'a>>,
}

/// An element of the structure of a document.
pub enum DocumentElement<'a> {
    Heading(u8, Option<String>, String),
    Paragraph(String),
    Links(Vec<LinksElement<'a>>),
}

/// An element of a links panel.
pub enum LinksElement<'a> {
    /// An inline heading within the links.
    Heading(u8, String, Option<String>),
    /// A link to an article and optional index.
    Link(&'a Article<'a>, Option<String>),
}

pub struct DirNode { // TODO: Node tree with dirs and documents
    pub(crate) dir_name: String,
    pub(crate) crumb: Option<String>,
}
