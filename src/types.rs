use std::cell::RefCell;
use std::rc::Weak;
use crate::article::Class;

pub enum ArticleMeta {
    Generic,
    Notion,
    Statement {
        uses: Vec<Weak<RefCell<Class>>>,
        defines: Vec<Weak<RefCell<Class>>>,
    },
    Proof {
        of: Vec<Weak<RefCell<Class>>>,
        uses: Vec<Weak<RefCell<Class>>>,
    },
    Problem {
        uses: Vec<Weak<RefCell<Class>>>,
    },
    Solution {
        of : Weak<RefCell<Class>>,
        uses: Vec<Weak<RefCell<Class>>>,
    },
}


