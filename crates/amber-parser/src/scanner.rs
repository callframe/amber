use amber_diagnostic::{
    Diagnostic,
    Listener,
    Severity,
};
use amber_source::{
    location::{
        Location,
        Offset,
    },
    source::Source,
};
use unicode_general_category::{
    GeneralCategory,
    get_general_category,
};

use crate::token::{
    Token,
    TokenType,
};

pub struct Scanner<'life> {
    source: &'life Source,
    cursor: u32,
    listener: &'life mut dyn Listener,
}

impl<'life> Scanner<'life> {
    pub fn new(source: &'life Source, listener: &'life mut dyn Listener) -> Self {
        Scanner {
            source,
            cursor: 0,
            listener,
        }
    }

    fn get_char(&self) -> Option<char> {
        match self.source.get_source().get(self.cursor as usize..) {
            Some(c) => c.chars().next(),
            None => None,
        }
    }

    fn advance_cursor(&mut self, c: char) {
        self.cursor += c.len_utf8() as u32;
    }

    fn make_location(&self, start: u32) -> Location {
        let total_offset = self.source.get_offset() + Offset(start);
        let len = self.cursor - start;
        Location::new(total_offset, len)
    }

    fn make_lexeme(&self, start: u32) -> Option<&'life str> {
        self.source.get_source().get(start as usize..self.cursor as usize)
    }

    fn make_token(&self, start: u32, r#type: TokenType) -> Token<'life> {
        let location = self.make_location(start);
        let lexeme = self.make_lexeme(start);
        Token::new(r#type, lexeme, location)
    }

    fn make_unknown_char(&mut self, start: u32) -> Token<'life> {
        let lexeme = self.make_lexeme(start).unwrap();

        let diagnostic = Diagnostic::builder()
            .severity(Severity::Error)
            .message(format!("Unknown character: {:?}", lexeme))
            .location(self.make_location(start))
            .build();

        self.listener.emit(diagnostic);
        self.make_token(start, TokenType::Illegal)
    }

    fn scan_identifier(&mut self) -> TokenType {
        loop {
            match self.get_char() {
                Some(c) => match get_general_category(c) {
                    GeneralCategory::UppercaseLetter
                    | GeneralCategory::LowercaseLetter
                    | GeneralCategory::TitlecaseLetter
                    | GeneralCategory::ModifierLetter
                    | GeneralCategory::OtherLetter
                    | GeneralCategory::DecimalNumber => {
                        self.advance_cursor(c);
                    }
                    _ => break,
                },
                None => break,
            }
        }

        TokenType::Identifier
    }

    fn scan_number(&mut self) -> TokenType {
        let mut seen_dot = false;

        loop {
            let Some(c) = self.get_char() else {
                break;
            };

            match get_general_category(c) {
                GeneralCategory::DecimalNumber => {
                    self.advance_cursor(c);
                }
                _ if !seen_dot && c == '.' => {
                    seen_dot = true;
                    self.advance_cursor(c);
                }
                _ => break,
            }
        }

        if seen_dot { TokenType::Float } else { TokenType::Integer }
    }
}

impl<'life> Iterator for Scanner<'life> {
    type Item = Token<'life>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.source.get_end_offset() <= self.cursor {
            return None;
        }

        let c = match self.get_char() {
            Some(c) => c,
            None => return None,
        };

        let start = self.cursor;
        match get_general_category(c) {
            GeneralCategory::UppercaseLetter
            | GeneralCategory::LowercaseLetter
            | GeneralCategory::TitlecaseLetter
            | GeneralCategory::ModifierLetter
            | GeneralCategory::OtherLetter => {
                self.advance_cursor(c);
                let r#type = self.scan_identifier();
                return Some(self.make_token(start, r#type));
            }

            GeneralCategory::DecimalNumber => {
                self.advance_cursor(c);
                let r#type = self.scan_number();
                return Some(self.make_token(start, r#type));
            }
            _ => {}
        }

        match c {
            _ => {
                self.advance_cursor(c);
                Some(self.make_unknown_char(start))
            }
        }
    }
}
