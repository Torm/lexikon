use std::rc::{Rc};

/// A relation: <left> is <right>
pub struct Relation {
    pub(crate) left: RelationClass,
    pub(crate) right: RelationClass,
}

/// A class partially (or fully) applied to some of its parameters.
#[derive(Clone)]
pub enum RelationClass {
    Name(Rc<str>),
    Qual {
        name: Rc<str>, arguments: Box<[RelationClass]>
    }
}
