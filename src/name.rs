use std::rc::Rc;
use crate::markup::Markup;

pub type Name = Vec<NameElement>;

pub enum NameElement {
    Name(Markup),
    Preposition(Markup),
    Parameter { markup: Markup, class: Rc<str> },
}
