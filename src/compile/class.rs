use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::mem::transmute;
use std::rc::Rc;
use typed_arena::Arena;
use crate::compile::model::{ArticleType, LinkType};
use crate::read::article::ArticleElement;

/// A class of articles in a project.
pub struct Class<'a> {
    pub(crate) key: Rc<str>,
    pub(crate) article_type: &'a ArticleType<'a>,
    pub(crate) articles: RefCell<Vec<&'a Article<'a>>>,
    pub(crate) links_out: RefCell<Vec<(&'a LinkType<'a>, Vec<&'a Class<'a>>)>>,
    pub(crate) links_in: RefCell<Vec<(&'a LinkType<'a>, Vec<&'a Class<'a>>)>>,
}

impl<'a> Class<'a> {

    pub(crate) fn new(key: &str, article_type: &'a ArticleType<'a>) -> Self {
        Self {
            key: Rc::from(key),
            article_type,
            articles: RefCell::new(vec![]),
            links_out: RefCell::new(vec![]),
            links_in: RefCell::new(vec![]),
        }
    }

    pub fn contains_article(&self, article_key: &str) -> bool {
        for article in self.articles.borrow().iter() {
            if article.key.borrow().as_str() == article_key {
                return true;
            }
        }
        false
    }

    pub fn get_article(&self, article_key: &str) -> Option<&'a Article<'a>> {
        for article in self.articles.borrow().iter() {
            if article.key.borrow().as_str() == article_key {
                return Some(article)
            }
        }
        None
    }

}

impl<'a> Class<'a> {

    /// Resolve an article in a class according to the resolution paths.
    ///
    /// Returns the first article if none was in any path.
    pub fn resolve(&self, paths: &[String]) -> &'a Article<'a> {
        for path in paths {
            for article in self.articles.borrow().iter() {
                if article.key.borrow().ends_with(&format!("@{}", path)) {
                    return article;
                }
            }
        }
        self.articles.borrow().get(0).unwrap()
    }

}

/// An article.
pub struct Article<'a> {
    pub(crate) class: &'a Class<'a>,
    pub(crate) key: RefCell<String>,
    pub(crate) names: Vec<String>,
    pub(crate) content: Vec<ArticleElement>,
}

impl<'a> Article<'a> {

    pub fn get_class(&self) -> &'a Class<'a> {
        self.class
    }

}

pub struct Classes<'a> {
    class_arena: Arena<Class<'a>>,
    article_arena: Arena<Article<'a>>,
    class_map: RefCell<HashMap<Rc<str>, &'a Class<'a>>>,
    article_map: RefCell<HashMap<Rc<str>, &'a Article<'a>>>,
}

impl<'a> Classes<'a> {

    pub fn get_class(&self, key: &str) -> Option<&'a Class<'a>> {
        self.class_map.borrow().get(key).copied()
    }

    pub fn get_article(&self, key: &str) -> Option<&'a Article<'a>> {
        self.article_map.borrow().get(key.into()).copied()
    }

    pub fn get_classes(&self) -> Ref<HashMap<Rc<str>, &'a Class<'a>>> {
        self.class_map.borrow()
    }

}

impl<'a> Classes<'a> {

    pub fn new() -> Self {
        Self {
            class_arena: Arena::new(),
            article_arena: Arena::new(),
            class_map: RefCell::new(HashMap::new()),
            article_map: RefCell::new(HashMap::new()),
        }
    }

    pub(crate) fn insert_class(&'a self, class: Class<'a>) -> &'a Class<'a> {
        if self.class_map.borrow().contains_key(class.key.as_ref()) {
            eprintln!("Class {} was already registered.", class.key.as_ref());
        }
        let class = self.class_arena.alloc(class);
        let class: &'a Class<'a> = unsafe { transmute(class) }; // TODO: Fix? Should be OK since 'a lives as long as the arena.
        self.class_map.borrow_mut().insert(class.key.clone(), class);
        class
    }

    pub(crate) fn insert_article(&'a self, article: Article<'a>) -> &'a Article<'a> {
        if self.article_map.borrow().contains_key(article.key.borrow().as_str()) {
            eprintln!("Article {} was already registered.", article.key.borrow().as_str());
        }
        let article = self.article_arena.alloc(article);
        let article: &'a Article<'a> = unsafe { transmute(article) }; // TODO: Fix? Should be OK since 'a lives as long as the arena.
        self.article_map.borrow_mut().insert(Rc::from(article.key.borrow().as_str()), article);
        article.class.articles.borrow_mut().push(article);
        article
    }

    /// Insert a link of specified type from a class to another.
    ///
    /// No change if link already exists.
    pub(crate) fn insert_link(&'a self, type_: &'a LinkType<'a>, from: &'a Class<'a>, to: &'a Class<'a>) {
        let mut links_out = from.links_out.borrow_mut();
        let mut linked_out = false;
        for (link_type, linked_classes) in links_out.iter_mut() {
            if *link_type as *const LinkType == type_ as *const LinkType {
                linked_classes.push(to);
                linked_out = true;
            }
        }
        if !linked_out {
            links_out.push((type_, vec![to]));
        }
        let mut links_in = to.links_in.borrow_mut();
        let mut linked_in = false;
        for (link_type, linked_classes) in links_in.iter_mut() {
            if *link_type as *const LinkType == type_ as *const LinkType {
                linked_classes.push(from);
                linked_in = true;
            }
        }
        if !linked_in {
            links_in.push((type_, vec![from]));
        }
    }

}
