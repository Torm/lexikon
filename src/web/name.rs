use crate::name::{Name, NameElement};

pub struct NameVariants {
    pub short_name: String,
    pub long_name: Option<String>,
}

impl NameVariants {

    pub fn get_full_html(&self) -> String {
        if let Some(s) = &self.parametrization {
            s.clone()
        } else {
            format!("<strong>{}</strong>", &self.name)
        }
    }

    pub fn get_short_html(&self) -> String {
        format!("<strong>{}</strong>", &self.name)
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

}

fn process_names(read_names: &[Name]) -> Result<Vec<Name>, String> {
    let mut names = vec![];
    for read_name in read_names {
        let mut is_parametrized = false;
        for e in read_name {
            if matches!(e, NameElement::Name(..) | NameElement::Parameter(..)) {
                is_parametrized = true;
                break;
            }
        }
        if is_parametrized {
            let mut name = String::new();
            let mut parametrization = String::new();
            for e in read_name {
                match e {
                    NameElement::Name(n) => {
                        name.push_str(n);
                        parametrization.push_str("<strong>");
                        parametrization.push_str(n);
                        parametrization.push_str("</strong>");
                    }
                    NameElement::Preposition(p) => {
                        parametrization.push_str(p);
                    }
                    NameElement::Parameter(p, k) => {
                        parametrization.push_str(&format!(r#"<b data-class="{}">"#, k));
                        parametrization.push_str(p);
                        parametrization.push_str("</b>");
                    }
                }
            }
            names.push(Name { name, parametrization: Some(parametrization) });
        } else {
            let mut name = String::new();
            for e in read_name {
                if let NameElement::Preposition(e) = e {
                    name.push_str(e);
                } else {
                    unreachable!()
                }
            }
            names.push(Name { name, parametrization: None });
        }
    }
    Ok(names)
}