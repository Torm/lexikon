use std::ffi::OsString;
use std::rc::{Rc, Weak};
use crate::dir::Dir;
use crate::markup::Markup;

pub type Documents = Vec<Rc<Document>>;

/// A processed document.
pub struct Document {
    pub(crate) key: String,
    pub(crate) parent_dir: Weak<Dir>,
    pub(crate) file_name: OsString, // Todo: Specific to reading from fs. But can be here for now since that is the only option.
    pub(crate) title: String,
    pub(crate) description: Option<String>,
//    pub(crate) localized_macros: Macros, // We don't need this since all macros are expanded into Markup during reading of document.
    pub(crate) resolution_paths: Vec<String>,
    pub(crate) structure: Vec<DocumentElement>,
}

impl Document {

    pub fn dirtrail(&self) -> Vec<Rc<Dir>> {
        let mut trail = vec![];
        trail.push(self.parent_dir.upgrade().unwrap());
        while let Some(parent) = &trail.last().unwrap().parent {
            trail.push(parent.upgrade().unwrap());
        }
        trail.reverse();
        trail
    }

}

//// Documents directory

/// An element of the structure of a document.
pub enum DocumentElement {
    Heading { level: u8, heading: Markup, index: Option<String> },
    Paragraph(Markup),
    Panel(Vec<PanelElement>),
}

/// An element of a links panel.
pub enum PanelElement {
    /// An inline heading within a panel.
    Heading { level: u8, heading: Markup, index: Option<String> },
    /// A link to an article and optional index.
    ArticleLink { key: Rc<str>, index: Option<String> },
    ClassLink { key: Rc<str>, index: Option<String> },
}

pub struct DirCrumb { // TODO: Node tree with dirs and documents?
    pub(crate) dir_name: String,
    pub(crate) crumb: Option<String>,
}
