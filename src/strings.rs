use std::collections::{HashSet};
use std::rc::Rc;

pub struct Strings(HashSet<Rc<str>>);

impl Strings {
    
    fn intern(&mut self, string: &str) -> Rc<str> {
        if let Some(interned) = self.0.get(string) {
            interned.clone()
        } else {
            let interned: Rc<str> = Rc::from(string);
            self.0.insert(interned.clone());
            interned
        }
    }
    
}
