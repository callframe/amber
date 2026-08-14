use amber_source::location::Location;

#[derive(Debug, Clone, Copy)]
pub enum TokenType {
    Illegal,
    Newline,

    // Literals
    Identifier,
    Integer,
    Float,

    // Brackets
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Modulo,

    // Assign Operators
    Assign,
    PlusAssign,
    MinusAssign,
    StarAssign,
    SlashAssign,

    // Comparison Operators
    Equal,
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
