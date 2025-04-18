//! Structures describing the knowledge model.

use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use typed_arena::Arena;

/// A collection of the article and link types used in a project.
pub struct Model<'a> {
    pub type_arena: Arena<ArticleType<'a>>,
    pub link_arena: Arena<LinkType<'a>>,
    pub type_map: RefCell<HashMap<String, &'a ArticleType<'a>>>,
}

impl<'a> Model<'a> {

    pub(crate) fn new() -> Self {
        Self {
            type_arena: Arena::new(),
            link_arena: Arena::new(),
            type_map: RefCell::new(HashMap::new()),
        }
    }

}

impl<'a> Model<'a> {

    pub fn get_type(&self, key: &str) -> Option<&'a ArticleType<'a>> {
        self.type_map.borrow().get(key).copied()
    }

}

/// An article type.
pub struct ArticleType<'a> {
    pub key: String,
    pub name: String,
    pub description: String,
    pub colour: Option<String>,
    pub abbreviation: Option<String>,
    pub symbol: Option<String>,
    pub links: RefCell<Vec<&'a LinkType<'a>>>,
}

impl<'a> ArticleType<'a> {

    pub fn get_link(&self, key: &str) -> Option<&'a LinkType<'a>> {
        for link in self.links.borrow().iter() {
            if link.key.as_str() == key {
                return Some(&link)
            }
        }
        None
    }

    pub fn get_links(&self) -> Ref<Vec<&'a LinkType<'a>>> {
        self.links.borrow()
    }

}

/// A link type.
pub struct LinkType<'a> {
    pub article_type: &'a ArticleType<'a>,
    pub key: String,
    pub origin_name: String,
    pub origin_description: String,
    pub target_name: String,
    pub target_description: String,
    pub target_show: bool,
}

impl<'a> LinkType<'a> {

    pub fn get_article_type(&self) -> &'a ArticleType<'a> {
        self.article_type
    }

}
