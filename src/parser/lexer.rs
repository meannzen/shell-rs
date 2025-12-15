use crate::error::ShellError;
use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
}

impl Token {
    pub fn tokenize(input: &str) -> Result<Vec<Token>, ShellError> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();

        while chars.peek().is_some() {
            if let Some(&c) = chars.peek()
                && c.is_whitespace()
            {
                chars.next();
                continue;
            }

            let token = Token::read_word(&mut chars);
            tokens.push(token);
        }

        Ok(tokens)
    }

    fn read_word(chars: &mut Peekable<Chars>) -> Token {
        let mut word = String::new();

        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                break;
            }
            if matches!(c, '|' | ';' | '>' | '<' | '&') {
                break;
            }

            word.push(c);
            chars.next();
        }

        Token::Word(word)
    }
}
