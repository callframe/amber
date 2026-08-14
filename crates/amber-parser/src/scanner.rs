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

    fn get_char(&self, offset: u32) -> Option<char> {
        let cursor = self.cursor + offset;
        self.source
            .get_source()
            .get(cursor as usize..)
            .and_then(|s| s.chars().next())
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
        while let Some(c) = self.get_char(0) {
            match get_general_category(c) {
                GeneralCategory::UppercaseLetter
                | GeneralCategory::LowercaseLetter
                | GeneralCategory::TitlecaseLetter
                | GeneralCategory::ModifierLetter
                | GeneralCategory::OtherLetter
                | GeneralCategory::DecimalNumber => {
                    self.advance_cursor(c);
                }
                _ => {
                    if c == '_' {
                        self.advance_cursor(c);
                    } else {
                        break;
                    }
                }
            }
        }

        TokenType::Identifier
    }

    fn scan_number(&mut self) -> TokenType {
        let mut seen_dot = false;

        while let Some(c) = self.get_char(0) {
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

    fn scan_comment(&mut self) {
        while let Some(c) = self.get_char(0) {
            match c {
                '\n' | '\r' => break,
                _ => self.advance_cursor(c),
            }
        }
    }

    fn scan_trivia(&mut self) -> Option<TokenType> {
        while let Some(c) = self.get_char(0) {
            match c {
                // TODO: consider whether we support docstrings
                '/' => {
                    if let Some('/') = self.get_char(1) {
                        self.advance_cursor(c);
                        self.advance_cursor('/');
                        self.scan_comment();
                    } else {
                        break;
                    }
                }

                '\r' => {
                    self.advance_cursor(c);
                    if let Some('\n') = self.get_char(0) {
                        self.advance_cursor('\n');
                    }
                    return Some(TokenType::Newline);
                }

                '\n' => {
                    self.advance_cursor(c);
                    return Some(TokenType::Newline);
                }

                ' ' | '\t' => {
                    self.advance_cursor(c);
                }

                _ => break,
            }
        }

        None
    }
}

impl<'life> Iterator for Scanner<'life> {
    type Item = Token<'life>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.source.get_end_offset() <= self.cursor {
            return None;
        }

        let start = self.cursor;
        if let Some(r#type) = self.scan_trivia() {
            return Some(self.make_token(start, r#type));
        }

        let c = self.get_char(0).unwrap();
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
            '_' => {
                self.advance_cursor(c);
                let r#type = self.scan_identifier();
                Some(self.make_token(start, r#type))
            }

            // Brackets
            '{' => {
                self.advance_cursor(c);
                Some(self.make_token(start, TokenType::LeftBrace))
            }

            '}' => {
                self.advance_cursor(c);
                Some(self.make_token(start, TokenType::RightBrace))
            }

            '(' => {
                self.advance_cursor(c);
                Some(self.make_token(start, TokenType::LeftParen))
            }

            ')' => {
                self.advance_cursor(c);
                Some(self.make_token(start, TokenType::RightParen))
            }

            '[' => {
                self.advance_cursor(c);
                Some(self.make_token(start, TokenType::LeftBracket))
            }

            ']' => {
                self.advance_cursor(c);
                Some(self.make_token(start, TokenType::RightBracket))
            }

            // Operators
            '+' => {
                self.advance_cursor(c);
                match self.get_char(0) {
                    Some('=') => {
                        self.advance_cursor('=');
                        Some(self.make_token(start, TokenType::PlusAssign))
                    }
                    _ => Some(self.make_token(start, TokenType::Plus)),
                }
            }

            '-' => {
                self.advance_cursor(c);
                match self.get_char(0) {
                    Some('=') => {
                        self.advance_cursor('=');
                        Some(self.make_token(start, TokenType::MinusAssign))
                    }
                    _ => Some(self.make_token(start, TokenType::Minus)),
                }
            }

            '*' => {
                self.advance_cursor(c);
                match self.get_char(0) {
                    Some('=') => {
                        self.advance_cursor('=');
                        Some(self.make_token(start, TokenType::StarAssign))
                    }
                    _ => Some(self.make_token(start, TokenType::Star)),
                }
            }

            '/' => {
                self.advance_cursor(c);
                match self.get_char(0) {
                    Some('=') => {
                        self.advance_cursor('=');
                        Some(self.make_token(start, TokenType::SlashAssign))
                    }
                    _ => Some(self.make_token(start, TokenType::Slash)),
                }
            }

            '%' => {
                self.advance_cursor(c);
                Some(self.make_token(start, TokenType::Modulo))
            }

            '=' => {
                self.advance_cursor(c);
                match self.get_char(0) {
                    Some('=') => {
                        self.advance_cursor('=');
                        Some(self.make_token(start, TokenType::Equal))
                    }
                    _ => Some(self.make_token(start, TokenType::Assign)),
                }
            }

            // Else
            _ => {
                self.advance_cursor(c);
                Some(self.make_unknown_char(start))
            }
        }
    }
}
