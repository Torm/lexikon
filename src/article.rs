use std::cell::{RefCell};
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::rc::{Rc, Weak};
use crate::relation::Relation;
use crate::markup::Markup;
use crate::name::Name;
use crate::types::ArticleMeta;

pub struct Articles {
    pub class_map: HashMap<Rc<str>, Rc<RefCell<Class>>>,
    pub article_map: HashMap<Rc<str>, Rc<RefCell<Article>>>,
}

impl Articles {

    pub fn new() -> Self {
        Self {
            class_map: HashMap::new(),
            article_map: HashMap::new(),
        }
    }

    pub fn get_class(&self, key: &str) -> Option<&Rc<RefCell<Class>>> {
        self.class_map.get(key)
    }

    pub fn get_article(&self, key: &str) -> Option<&Rc<RefCell<Article>>> {
        self.article_map.get(key)
    }

    pub fn get_classes(&self) -> &HashMap<Rc<str>, Rc<RefCell<Class>>> {
        &self.class_map
    }

    pub fn get_articles(&self) -> &HashMap<Rc<str>, Rc<RefCell<Article>>> {
        &self.article_map
    }

}

/// A class of articles in a project.
pub struct Class {
    pub(crate) key: Rc<str>,
    pub(crate) parameters: Parameters,
    /// Articles under this class.
    pub(crate) articles: Vec<Weak<RefCell<Article>>>, // TODO: Strong not weak
    /// Relations relevant to this class.
    pub(crate) relations: HashSet<Relation>,
    pub style: Option<Rc<str>>,
}

pub type Parameters = Box<[Rc<str>]>; // TODO: Weak<RefCell<Class>> instead of Rc<str>

/// Verify that two lists of parameters match.
pub fn verify_parameter_match(p1: &Parameters, p2: &Parameters) -> Result<(), String> {
    if p1.len() != p2.len() {
        return Err(format!("Number of parameters defined in article does not match number in class."));
    }
    let mut i = 0;
    while i < p1.len() {
        let p1 = &p1[i];
        let p2 = &p2[i];
        if p1.as_ptr() != p2.as_ptr() {
            return Err(format!("A specific parameter of article and class does not match."));
        }
        i += 1;
    }
    Ok(())
}

// // TODO: Need this?
// pub struct PropertyConfig {
//     property_name: String,
//     property_description: String,
//     value_name: String,
//     value_description: String,
// }

impl Class {

    pub(crate) fn new(key: &str, ) -> Self {
        Self {
            key: Rc::from(key),
            parameters: Box::new([]),
            articles: vec![],
            relations: HashSet::new(),
            style: None,
        }
    }

    /// Resolve an article in a class according to the resolution paths.
    ///
    /// Returns the first article if none was in any path.
    pub fn resolve(&self, paths: &[String]) -> Rc<RefCell<Article>> {
        for path in paths {
            for article_ref in self.articles.iter() {
                let article = article_ref.upgrade().unwrap();
                let article = article.borrow();
                if article.key.ends_with(&format!("@{}", path)) {
                    return article_ref.upgrade().unwrap();
                }
            }
        }
        self.articles.get(0).unwrap().upgrade().unwrap()
    }

    pub fn contains_article(&self, article_key: &str) -> bool {
        for article in self.articles.iter() {
            let article = article.upgrade().unwrap();
            let article = article.borrow();
            if article.key.deref() == article_key {
                return true;
            }
        }
        false
    }

    pub fn get_article(&self, article_key: &str) -> Option<Rc<RefCell<Article>>> {
        for article in self.articles.iter() {
            let article = article.upgrade().unwrap();
            {
                let article_ref = article.borrow();
                if article_ref.key.deref() != article_key {
                    continue;
                }
            }
            return Some(article)
        }
        None
    }

}

/// An article.
pub struct Article {
    pub(crate) key: Rc<str>,
    pub(crate) class: Weak<RefCell<Class>>,
    pub(crate) names: Vec<Name>,
    pub(crate) content: Vec<ArticleElement>,
    pub(crate) metadata: ArticleMeta,
}

/// Element of article content.
#[derive(Clone)]
pub enum ArticleElement {
    Heading { level: u8, markup: Markup },
    Markup(Markup),
    /// Line that indicates separation between two instances of an article in the document.
    LocalSeparator,
}

impl Article {

    pub fn get_class(&self) -> Rc<RefCell<Class>> {
        self.class.upgrade().unwrap()
    }

}
