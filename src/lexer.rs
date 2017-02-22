//! This module contains the lexer for Kaleidoscope.

// The lexer will be implemented as an iterator, so we need to use the Iterator trait.
use std::iter::Iterator;
use std::iter::Peekable;
// The lexer will use the Chars type.
use std::str::Chars;

/// All the different tokens that the lexer can return.
///
// Using Rust enums instead of integers is much safer and more readable.
#[derive(Debug, PartialEq)]
pub enum Token {
    // Commands
    Define,
    Extern,
    /// An Identifier contains the identifier as a String.
    /// This is much safer and easier to manage than using global variables.
    Identifier(String),
    /// All numbers in Kaleidoscope are 64 bit floats.
    /// We store the number in the variant istead of in a global variable
    /// for the same reasons as Identifier.
    Number(f64),
    /// UnknownChar corresponds to returning a positive integer from gettok.
    UnknownChar(char),
}

/// The lexer is implemented as a struct that holds its state instead of a
/// function that works on global state because it is more general and easier to use.
#[derive(Debug)]
pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    source: &'a str,
}

impl<'a> Lexer<'a> {
    /// The constructor for Lexer.
    pub fn new(source: &'a str) -> Lexer<'a> {
        Lexer {
            chars: source.chars().peekable(),
            source: source,
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    // We will be iterating over Tokens
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        let mut next = self.chars.next();
        while let Some(c) = next {
            if !c.is_whitespace() {
                break;
            }
            next = self.chars.next();
        }
        if let Some(c) = next {
            if c.is_alphabetic() {
                let mut identifier = String::new();
                identifier.push(c);
                loop {
                    // We create a new block so that x will be out of scope when
                    // self.chars.next() is called.
                    // This avoids a multiple mutable reference error
                    {
                        let x = self.chars.peek();
                        match x {
                            Some(c) if c.is_alphanumeric() => identifier.push(*c),
                            _ => break,
                        }
                    };
                    self.chars.next();
                }
                if identifier == "def" {
                    Some(Token::Define)
                } else if identifier == "extern" {
                    Some(Token::Extern)
                } else {
                    Some(Token::Identifier(identifier))
                }
            } else if c.is_digit(10) || c == '.' {
                let mut num = String::new();
                num.push(c);
                loop {
                    // We create a new block so that x will be out of scope when
                    // self.chars.next() is called.
                    // This avoids a multiple mutable reference error
                    {
                        let x = self.chars.peek();
                        match x {
                            Some(c) if c.is_digit(10) || *c == '.' => num.push(*c),
                            _ => break,
                        }
                    };
                    self.chars.next();
                }
                Some(Token::Number(num.parse().expect("Could not parse number!")))
            } else if c == '#' {
                loop {
                    // We create a new block so that x will be out of scope when
                    // self.chars.next() is called.
                    // This avoids a multiple mutable reference error
                    {
                        let x = self.chars.peek();
                        match x {
                            // Just eat the chars
                            Some(c) if *c != '\r' && *c != '\n' => {},
                            _ => break,
                        }
                    };
                    self.chars.next();
                }
                self.next()
            } else {
                Some(Token::UnknownChar(c))
            }
        } else {
            None
        }
    }
}

// Some tests for the lexer
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_simple_tokens_and_value() {
        let mut lexer = Lexer::new("1 + 1 - foo");
        assert_eq!(lexer.next().unwrap(), Token::Number(1.0));
        assert_eq!(lexer.next().unwrap(), Token::UnknownChar('+'));
        assert_eq!(lexer.next().unwrap(), Token::Number(1.0));
        assert_eq!(lexer.next().unwrap(), Token::UnknownChar('-'));
        assert_eq!(lexer.next().unwrap(), Token::Identifier(String::from("foo")));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn test_simple_tokens_and_value_no_whitespace() {
        let mut lexer = Lexer::new("1+1-foo");
        assert_eq!(lexer.next().unwrap(), Token::Number(1.0));
        assert_eq!(lexer.next().unwrap(), Token::UnknownChar('+'));
        assert_eq!(lexer.next().unwrap(), Token::Number(1.0));
        assert_eq!(lexer.next().unwrap(), Token::UnknownChar('-'));
        assert_eq!(lexer.next().unwrap(), Token::Identifier(String::from("foo")));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn test_comments() {
        let code = "# This is a comment 1+1
        1 + 2 # <- is code
        # this is not";
        let mut lexer = Lexer::new(code);
        assert_eq!(lexer.next(), Some(Token::Number(1.0)));
        assert_eq!(lexer.next(), Some(Token::UnknownChar('+')));
        assert_eq!(lexer.next(), Some(Token::Number(2.0)));
        assert_eq!(lexer.next(), None);
    }
}
