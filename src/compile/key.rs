use std::iter::Peekable;
use std::str::Chars;

fn is_key_character(char: char) -> bool{
    char.is_ascii_alphanumeric() || char == '-' || char == '\'' || char == '&' || char == '.' || char == '#'
}

pub struct KeyReader<'a> {
    key: Peekable<Chars<'a>>,
}

impl<'a> KeyReader<'a> {

    pub fn new(key_str: &'a str) -> Self {
        Self { key: key_str.chars().peekable() }
    }

    pub fn is_plain_key(&mut self) -> bool {
        if let Some(c) = self.key.peek() {
            is_key_character(*c)
        } else {
            false
        }
    }

    /// Parse a plain key.
    ///
    /// Assumes [Self::is_plain_key] is true.
    ///
    /// The bool is true if this is an article key, and false if this is a
    /// class key.
    pub fn parse_plain(&mut self) -> Result<(String, bool), String> {
        let mut result = String::new();
        let mut article_key = false;
        while let Some(c) = self.key.peek() {
            if is_key_character(*c) {
                result.push(*c);
                self.key.next();
            } else if *c == '@' {
                if article_key {
                    return Err(format!("Cannot have multiple @ characters in key."));
                }
                result.push('@');
                article_key = true;
                self.key.next();
                if !self.is_plain_key() {
                    return Err(format!("Article key missing article after @ character."));
                }
            } else {
                break;
            }
        }
        Ok((result, article_key))
    }

    pub fn is_parenthesized(&mut self) -> bool {
        if let Some(c) = self.key.peek() {
            *c == '('
        } else {
            false
        }
    }

    /// Parse a parenthesized key.
    ///
    /// Assumes [Self::is_parenthesized] is true.
    pub fn parse_parenthesized(&mut self) -> Result<String, String> {
        self.key.next();
        if !self.is_plain_key() {
            return Err(format!("Expected key in parentheses."));
        }
        let (key, article) = self.parse_plain()?;
        if article {
            return Err(format!("Character '@' is not allowed in local key."));
        }
        if let Some(c) = self.key.peek() {
            if *c != ')' {
                return Err(format!("Expected ')', found '{}'", c));
            }
        } else {
            return Err(format!("Expected ')', found end of key."));
        }
        self.key.next();
        Ok(key)
    }

    pub fn is_at_end(&mut self) -> bool {
        self.key.peek().is_none()
    }

    pub fn skip_whitespace(&mut self) -> Result<(), String> {
        while let Some(c) = self.key.peek() {
            if *c == ' ' || *c == '\t' {
                break;
            } else {
                self.key.next();
            }
        }
        Ok(())
    }

}
