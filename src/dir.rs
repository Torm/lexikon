use std::ffi::OsString;
use std::rc::{Rc, Weak};
use crate::document::Document;

pub struct Dir {
    pub(crate) name: String,
    pub(crate) file_name: OsString,
    pub(crate) parent: Option<Weak<Dir>>,
    pub(crate) subdirs: Vec<Rc<Dir>>,
    pub(crate) subdocs: Vec<Rc<Document>>,
}

impl Dir {

    pub fn dirtrail(self: &Rc<Self>) -> Vec<Rc<Dir>> {
        let mut trail = vec![];
        trail.push(self.clone());
        while let Some(parent) = &trail.last().unwrap().parent {
            trail.push(parent.upgrade().unwrap().clone());
        }
        trail.reverse();
        trail
    }

}


