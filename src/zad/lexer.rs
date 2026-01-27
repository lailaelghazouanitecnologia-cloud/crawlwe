//! ZAD Lexer - Tokenizes ZAD source code

use crate::zad::{ZadError, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Template,
    Import,
    From,
    Const,
    Let,
    If,
    Else,
    For,
    Switch,
    True,
    False,
    Null,

    // Identifiers and literals
    Ident(String),
    String(String),
    Int(i64),
    Float(f64),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    EqEq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
    Dot,
    DotDot,
    Concat,     // ++
    Arrow,      // =>
    Pipe,       // |

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Semi,
    At,         // @
    Question,   // ?

    // Special
    Interpolation(String),  // {expr} in template
    Text(String),           // Raw text in template
    Newline,
    Eof,
}

impl Token {
    pub fn is_keyword(s: &str) -> Option<Token> {
        match s {
            "template" => Some(Token::Template),
            "import" => Some(Token::Import),
            "from" => Some(Token::From),
            "const" => Some(Token::Const),
            "let" => Some(Token::Let),
            "if" => Some(Token::If),
            "else" => Some(Token::Else),
            "for" => Some(Token::For),
            "switch" => Some(Token::Switch),
            "true" => Some(Token::True),
            "false" => Some(Token::False),
            "null" => Some(Token::Null),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub token: Token,
    pub line: usize,
    pub col: usize,
}

pub struct Lexer<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    line: usize,
    col: usize,
    in_template: bool,
    brace_depth: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
            line: 1,
            col: 1,
            in_template: false,
            brace_depth: 0,
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<TokenInfo>> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let is_eof = token.token == Token::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<TokenInfo> {
        self.skip_whitespace();

        let line = self.line;
        let col = self.col;

        let Some((pos, ch)) = self.chars.next() else {
            return Ok(TokenInfo { token: Token::Eof, line, col });
        };

        self.col += 1;

        let token = match ch {
            // Single char tokens
            '(' => Token::LParen,
            ')' => Token::RParen,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            ',' => Token::Comma,
            ':' => Token::Colon,
            ';' => Token::Semi,
            '@' => Token::At,
            '?' => Token::Question,
            '*' => Token::Star,
            '%' => Token::Percent,

            // Braces (track depth for template mode)
            '{' => {
                if self.in_template && self.brace_depth == 0 {
                    // Start of interpolation
                    self.brace_depth += 1;
                    Token::LBrace
                } else {
                    self.brace_depth += 1;
                    Token::LBrace
                }
            }
            '}' => {
                self.brace_depth = self.brace_depth.saturating_sub(1);
                Token::RBrace
            }

            // Two-char tokens
            '+' => {
                if self.peek_char() == Some('+') {
                    self.advance();
                    Token::Concat
                } else {
                    Token::Plus
                }
            }
            '-' => Token::Minus,
            '/' => {
                if self.peek_char() == Some('/') {
                    // Line comment
                    self.skip_line_comment();
                    return self.next_token();
                } else if self.peek_char() == Some('*') {
                    // Block comment
                    self.skip_block_comment()?;
                    return self.next_token();
                } else {
                    Token::Slash
                }
            }
            '=' => {
                if self.peek_char() == Some('=') {
                    self.advance();
                    Token::EqEq
                } else if self.peek_char() == Some('>') {
                    self.advance();
                    Token::Arrow
                } else {
                    Token::Eq
                }
            }
            '!' => {
                if self.peek_char() == Some('=') {
                    self.advance();
                    Token::Ne
                } else {
                    Token::Not
                }
            }
            '<' => {
                if self.peek_char() == Some('=') {
                    self.advance();
                    Token::Le
                } else {
                    Token::Lt
                }
            }
            '>' => {
                if self.peek_char() == Some('=') {
                    self.advance();
                    Token::Ge
                } else {
                    Token::Gt
                }
            }
            '&' => {
                if self.peek_char() == Some('&') {
                    self.advance();
                    Token::And
                } else {
                    return Err(ZadError::LexerError {
                        line,
                        message: "Expected '&&'".into(),
                    });
                }
            }
            '|' => {
                if self.peek_char() == Some('|') {
                    self.advance();
                    Token::Or
                } else {
                    Token::Pipe
                }
            }
            '.' => {
                if self.peek_char() == Some('.') {
                    self.advance();
                    Token::DotDot
                } else {
                    Token::Dot
                }
            }

            // Strings
            '"' => self.read_string('"')?,
            '\'' => self.read_string('\'')?,
            '`' => self.read_template_string()?,

            // Newline
            '\n' => {
                self.line += 1;
                self.col = 1;
                Token::Newline
            }

            // Numbers
            c if c.is_ascii_digit() => self.read_number(pos, c)?,

            // Identifiers
            c if c.is_alphabetic() || c == '_' => self.read_ident(pos, c),

            _ => {
                return Err(ZadError::LexerError {
                    line,
                    message: format!("Unexpected character: '{}'", ch),
                });
            }
        };

        Ok(TokenInfo { token, line, col })
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, c)| *c)
    }

    fn advance(&mut self) -> Option<char> {
        self.chars.next().map(|(_, c)| {
            self.col += 1;
            c
        })
    }

    fn skip_whitespace(&mut self) {
        while let Some(&(_, ch)) = self.chars.peek() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.advance() {
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
                break;
            }
        }
    }

    fn skip_block_comment(&mut self) -> Result<()> {
        self.advance(); // consume *
        let start_line = self.line;

        loop {
            match self.advance() {
                Some('*') => {
                    if self.peek_char() == Some('/') {
                        self.advance();
                        return Ok(());
                    }
                }
                Some('\n') => {
                    self.line += 1;
                    self.col = 1;
                }
                Some(_) => {}
                None => {
                    return Err(ZadError::LexerError {
                        line: start_line,
                        message: "Unterminated block comment".into(),
                    });
                }
            }
        }
    }

    fn read_string(&mut self, quote: char) -> Result<Token> {
        let mut s = String::new();
        let start_line = self.line;

        loop {
            match self.advance() {
                Some(c) if c == quote => {
                    return Ok(Token::String(s));
                }
                Some('\\') => {
                    match self.advance() {
                        Some('n') => s.push('\n'),
                        Some('r') => s.push('\r'),
                        Some('t') => s.push('\t'),
                        Some('\\') => s.push('\\'),
                        Some(c) if c == quote => s.push(quote),
                        Some(c) => s.push(c),
                        None => {
                            return Err(ZadError::LexerError {
                                line: start_line,
                                message: "Unterminated string".into(),
                            });
                        }
                    }
                }
                Some('\n') => {
                    self.line += 1;
                    self.col = 1;
                    s.push('\n');
                }
                Some(c) => s.push(c),
                None => {
                    return Err(ZadError::LexerError {
                        line: start_line,
                        message: "Unterminated string".into(),
                    });
                }
            }
        }
    }

    fn read_template_string(&mut self) -> Result<Token> {
        // For now, treat backtick strings as regular strings
        // TODO: Parse interpolations
        let mut s = String::new();
        let start_line = self.line;

        loop {
            match self.advance() {
                Some('`') => {
                    return Ok(Token::String(s));
                }
                Some('\\') => {
                    match self.advance() {
                        Some(c) => s.push(c),
                        None => {
                            return Err(ZadError::LexerError {
                                line: start_line,
                                message: "Unterminated template string".into(),
                            });
                        }
                    }
                }
                Some('\n') => {
                    self.line += 1;
                    self.col = 1;
                    s.push('\n');
                }
                Some(c) => s.push(c),
                None => {
                    return Err(ZadError::LexerError {
                        line: start_line,
                        message: "Unterminated template string".into(),
                    });
                }
            }
        }
    }

    fn read_number(&mut self, start_pos: usize, first: char) -> Result<Token> {
        let mut end_pos = start_pos + first.len_utf8();
        let mut is_float = false;

        while let Some(&(pos, ch)) = self.chars.peek() {
            if ch.is_ascii_digit() {
                end_pos = pos + ch.len_utf8();
                self.advance();
            } else if ch == '.' && !is_float {
                // Check if next is digit (not ..)
                let chars_clone = self.chars.clone();
                let mut iter = chars_clone.skip(1);
                if let Some((_, next_ch)) = iter.next() {
                    if next_ch.is_ascii_digit() {
                        is_float = true;
                        end_pos = pos + ch.len_utf8();
                        self.advance();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else if ch == '_' {
                // Allow underscores in numbers
                self.advance();
            } else {
                break;
            }
        }

        let num_str: String = self.source[start_pos..end_pos]
            .chars()
            .filter(|c| *c != '_')
            .collect();

        if is_float {
            let f: f64 = num_str.parse().map_err(|_| ZadError::LexerError {
                line: self.line,
                message: format!("Invalid float: {}", num_str),
            })?;
            Ok(Token::Float(f))
        } else {
            let i: i64 = num_str.parse().map_err(|_| ZadError::LexerError {
                line: self.line,
                message: format!("Invalid integer: {}", num_str),
            })?;
            Ok(Token::Int(i))
        }
    }

    fn read_ident(&mut self, start_pos: usize, first: char) -> Token {
        let mut end_pos = start_pos + first.len_utf8();

        while let Some(&(pos, ch)) = self.chars.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                end_pos = pos + ch.len_utf8();
                self.advance();
            } else {
                break;
            }
        }

        let ident = &self.source[start_pos..end_pos];

        // Check for keywords
        Token::is_keyword(ident).unwrap_or_else(|| Token::Ident(ident.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let source = "template foo(x: str) { }";
        let lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0].token, Token::Template));
        assert!(matches!(&tokens[1].token, Token::Ident(s) if s == "foo"));
        assert!(matches!(tokens[2].token, Token::LParen));
    }

    #[test]
    fn test_numbers() {
        let source = "42 3.14 1_000_000";
        let lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0].token, Token::Int(42)));
        assert!(matches!(tokens[1].token, Token::Float(f) if (f - 3.14).abs() < 0.001));
        assert!(matches!(tokens[2].token, Token::Int(1_000_000)));
    }

    #[test]
    fn test_strings() {
        let source = r#""hello" 'world'"#;
        let lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(&tokens[0].token, Token::String(s) if s == "hello"));
        assert!(matches!(&tokens[1].token, Token::String(s) if s == "world"));
    }
}
