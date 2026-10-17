#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    String(String),
    Number(f64),
    Identifier(String),
    
    // Keywords
    State,
    View,
    Update,
    When,
    If,
    Then,
    Else,
    Let,
    In,
    True,
    False,
    Null,
    
    // Operators
    Plus,           // +
    Minus,          // -
    Star,           // *
    Slash,          // /
    Percent,        // %
    Bang,           // !
    EqualEqual,     // ==
    BangEqual,      // !=
    Less,           // <
    LessEqual,      // <=
    Greater,        // >
    GreaterEqual,   // >=
    And,            // &&
    Or,             // ||
    Equal,          // =
    
    // Delimiters
    LParen,         // (
    RParen,         // )
    LBrace,         // {
    RBrace,         // }
    LBracket,       // [
    RBracket,       // ]
    
    // Punctuation
    Comma,          // ,
    Dot,            // .
    Colon,          // :
    Semicolon,      // ;
    Arrow,          // ->  (thin arrow)
    FatArrow,       // =>  (fat arrow)
    Spread,         // ...
    
    // Special
    Newline,
    Eof,
    Hash,   // # — used for style id-selectors and hex colours
}

#[derive(Debug, Clone)]
pub enum LexError {
    UnexpectedCharacter(char, usize, usize),
    InvalidNumber(String, usize, usize),
    UnterminatedString(usize, usize),
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LexError::UnexpectedCharacter(ch, line, col) => {
                write!(f, "Unexpected character '{}' at line {}, column {}", ch, line, col)
            }
            LexError::InvalidNumber(num, line, col) => {
                write!(f, "Invalid number '{}' at line {}, column {}", num, line, col)
            }
            LexError::UnterminatedString(line, col) => {
                write!(f, "Unterminated string at line {}, column {}", line, col)
            }
        }
    }
}

impl std::error::Error for LexError {}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current = chars.first().copied();
        Self {
            input: chars,
            position: 0,
            current_char: current,
            line: 1,
            column: 1,
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        
        loop {
            match self.next_token()? {
                Token::Eof => {
                    tokens.push(Token::Eof);
                    break;
                }
                Token::Newline => continue,
                token => tokens.push(token),
            }
        }
        
        Ok(tokens)
    }
    
    fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace_and_comments();
        
        let ch = match self.current_char {
            Some(c) => c,
            None => return Ok(Token::Eof),
        };
        
        if ch.is_ascii_digit() {
            return self.read_number();
        }
        
        if ch.is_alphabetic() || ch == '_' {
            return self.read_identifier_or_keyword();
        }
        
        if ch == '"' {
            return self.read_string();
        }
        
        if ch == '#' {
            return self.read_hash_token();
        }
        
        match ch {
            '=' => {
                self.advance();
                if self.current_char == Some('>') {
                    self.advance();
                    return Ok(Token::FatArrow);
                }
                if self.current_char == Some('=') {
                    self.advance();
                    return Ok(Token::EqualEqual);
                }
                return Ok(Token::Equal);
            }
            '!' => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    return Ok(Token::BangEqual);
                }
                return Ok(Token::Bang);
            }
            '<' => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    return Ok(Token::LessEqual);
                }
                return Ok(Token::Less);
            }
            '>' => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    return Ok(Token::GreaterEqual);
                }
                return Ok(Token::Greater);
            }
            '&' => {
                self.advance();
                if self.current_char == Some('&') {
                    self.advance();
                    return Ok(Token::And);
                }
                return Err(LexError::UnexpectedCharacter('&', self.line, self.column));
            }
            '|' => {
                self.advance();
                if self.current_char == Some('|') {
                    self.advance();
                    return Ok(Token::Or);
                }
                return Err(LexError::UnexpectedCharacter('|', self.line, self.column));
            }
            '-' => {
                self.advance();
                if self.current_char == Some('>') {
                    self.advance();
                    return Ok(Token::Arrow);  // -> is Arrow
                }
                return Ok(Token::Minus);
            }
            '.' => {
                self.advance();
                if self.current_char == Some('.') && self.peek(1) == Some('.') {
                    self.advance();
                    self.advance();
                    return Ok(Token::Spread);
                }
                return Ok(Token::Dot);
            }
            _ => {}
        }
        
        let token = match ch {
            '+' => Token::Plus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            ',' => Token::Comma,
            ':' => Token::Colon,
            ';' => Token::Semicolon,
            '\n' => Token::Newline,
            _ => return Err(LexError::UnexpectedCharacter(ch, self.line, self.column)),
        };
        
        self.advance();
        Ok(token)
    }
    
    fn advance(&mut self) {
        if self.current_char == Some('\n') {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }
    
    fn peek(&self, offset: usize) -> Option<char> {
        self.input.get(self.position + offset).copied()
    }
    
    fn skip_whitespace_and_comments(&mut self) {
        while let Some(ch) = self.current_char {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
                continue;
            }
            
            if ch == '/' && self.peek(1) == Some('/') {
                self.advance();
                self.advance();
                while let Some(c) = self.current_char {
                    if c == '\n' { break; }
                    self.advance();
                }
                continue;
            }
            
            if ch == '/' && self.peek(1) == Some('*') {
                self.advance();
                self.advance();
                while let Some(c) = self.current_char {
                    if c == '*' && self.peek(1) == Some('/') {
                        self.advance();
                        self.advance();
                        break;
                    }
                    self.advance();
                }
                continue;
            }
            
            break;
        }
    }
    
    fn read_number(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.column;
        let mut num_str = String::new();
        let mut has_dot = false;
        
        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else if ch == '.' && !has_dot {
                if let Some(next) = self.peek(1) {
                    if next.is_ascii_digit() {
                        has_dot = true;
                        num_str.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        num_str
            .parse::<f64>()
            .map(Token::Number)
            .map_err(|_| LexError::InvalidNumber(num_str, start_line, start_col))
    }
    
    fn read_identifier_or_keyword(&mut self) -> Result<Token, LexError> {
        let mut ident = String::new();
        
        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        let token = match ident.as_str() {
            "state" => Token::State,
            "view" => Token::View,
            "update" => Token::Update,
            "when" => Token::When,
            "if" => Token::If,
            "then" => Token::Then,
            "else" => Token::Else,
            "let" => Token::Let,
            "in" => Token::In,
            "true" => Token::True,
            "false" => Token::False,
            "null" => Token::Null,
            _ => Token::Identifier(ident),
        };
        
        Ok(token)
    }
    
    // A `#` is either:
    //   • a hex colour like `#ff5500` or `#fff`  → Token::String("#ff5500")
    //   • a bare `#` for id-selectors in style rules  → Token::Hash
    fn read_hash_token(&mut self) -> Result<Token, LexError> {
        self.advance(); // consume '#'
        // Count consecutive hex digits starting at the current position
        // WITHOUT advancing — we're just peeking here.
        let hex_run: usize = self.input[self.position..]
            .iter()
            .take(7)                          // only need to know if we have 3 or 6
            .take_while(|&&c| c.is_ascii_hexdigit())
            .count();
        // Check what comes immediately after those hex digits.
        let after = self.input.get(self.position + hex_run).copied();
        let next_is_word_char = after
            .map(|c| c.is_alphanumeric() || c == '_' || c == '-')
            .unwrap_or(false);

        // Treat as a colour only when we have exactly 3 or 6 hex digits
        // followed by something that is NOT an identifier/word character.
        if (hex_run == 3 || hex_run == 6) && !next_is_word_char {
            let mut hex = String::from("#");
            for _ in 0..hex_run {
                hex.push(self.current_char.unwrap());
                self.advance();
            }
            Ok(Token::String(hex))
        } else {
            // Not a colour — return `#` as a bare hash token so the parser
            // can use it as an id-selector prefix.
            Ok(Token::Hash)
        }
    }

    fn read_string(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.column;
        
        self.advance(); // Skip opening "
        
        let mut string = String::new();
        
        while let Some(ch) = self.current_char {
            match ch {
                '"' => {
                    self.advance();
                    return Ok(Token::String(string));
                }
                '\\' => {
                    self.advance();
                    match self.current_char {
                        Some('n') => string.push('\n'),
                        Some('t') => string.push('\t'),
                        Some('r') => string.push('\r'),
                        Some('\\') => string.push('\\'),
                        Some('"') => string.push('"'),
                        Some('0') => string.push('\0'),
                        Some(c) => string.push(c),
                        None => return Err(LexError::UnterminatedString(start_line, start_col)),
                    }
                    self.advance();
                }
                '\n' => {
                    string.push(ch);
                    self.advance();
                }
                _ => {
                    string.push(ch);
                    self.advance();
                }
            }
        }
        
        Err(LexError::UnterminatedString(start_line, start_col))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_tokens() {
        let mut lexer = Lexer::new("{ : }");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0], Token::LBrace);
        assert_eq!(tokens[1], Token::Colon);
        assert_eq!(tokens[2], Token::RBrace);
    }
    
    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("state view update when if then else");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0], Token::State);
        assert_eq!(tokens[1], Token::View);
        assert_eq!(tokens[2], Token::Update);
        assert_eq!(tokens[3], Token::When);
        assert_eq!(tokens[4], Token::If);
        assert_eq!(tokens[5], Token::Then);
        assert_eq!(tokens[6], Token::Else);
    }
    
    #[test]
    // 3.14 is a decimal literal under test, not an approximation of PI.
    #[allow(clippy::approx_constant)]
    fn test_numbers() {
        let mut lexer = Lexer::new("42 3.14 0 100.0");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0], Token::Number(42.0));
        assert_eq!(tokens[1], Token::Number(3.14));
        assert_eq!(tokens[2], Token::Number(0.0));
        assert_eq!(tokens[3], Token::Number(100.0));
    }
    
    #[test]
    fn test_strings() {
        let mut lexer = Lexer::new(r#""hello" "world""#);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0], Token::String("hello".to_string()));
        assert_eq!(tokens[1], Token::String("world".to_string()));
    }
    
    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("+ - * / == != < <= > >= && ||");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0], Token::Plus);
        assert_eq!(tokens[1], Token::Minus);
        assert_eq!(tokens[2], Token::Star);
        assert_eq!(tokens[3], Token::Slash);
        assert_eq!(tokens[4], Token::EqualEqual);
        assert_eq!(tokens[5], Token::BangEqual);
        assert_eq!(tokens[6], Token::Less);
        assert_eq!(tokens[7], Token::LessEqual);
        assert_eq!(tokens[8], Token::Greater);
        assert_eq!(tokens[9], Token::GreaterEqual);
        assert_eq!(tokens[10], Token::And);
        assert_eq!(tokens[11], Token::Or);
    }
}
