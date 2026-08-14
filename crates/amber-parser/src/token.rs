use amber_source::location::Location;

#[derive(Debug, Clone, Copy)]
pub enum TokenType {
    Illegal,

    Identifier,
    Integer,
    Float,
}

#[derive(Debug, Clone)]
pub struct Token<'life> {
    r#type: TokenType,
    lexeme: Option<&'life str>,
    location: Location,
}

impl<'life> Token<'life> {
    pub fn new(r#type: TokenType, lexeme: Option<&'life str>, location: Location) -> Self {
        Token {
            r#type,
            lexeme,
            location,
        }
    }

    pub fn get_type(&self) -> TokenType {
        self.r#type
    }

    pub fn get_lexeme(&self) -> Option<&'life str> {
        self.lexeme
    }

    pub fn get_location(&self) -> Location {
        self.location
    }
}
