use crate::error::ShellError;
use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    Space,
}

impl Token {
    pub fn tokenize(input: &str) -> Result<Vec<Token>, ShellError> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
                tokens.push(Token::Space);
                continue;
            }

            if matches!(c, '|' | ';' | '>' | '<' | '&') {
                tokens.push(Token::Word(c.to_string()));
                chars.next();
                continue;
            }

            let token = Token::read_word(&mut chars)?;
            if let Token::Word(word) = &token
                && !word.is_empty()
            {
                tokens.push(token);
            }
        }

        Ok(tokens)
    }

    fn read_word(chars: &mut Peekable<Chars>) -> Result<Token, ShellError> {
        let mut word = String::new();

        while let Some(&c) = chars.peek() {
            if c == '\'' {
                chars.next();

                let mut found_closing = false;
                while let Some(&ch) = chars.peek() {
                    if ch == '\'' {
                        chars.next();
                        found_closing = true;
                        break;
                    }
                    word.push(ch);
                    chars.next();
                }

                if !found_closing {
                    return Err(ShellError::ParseError("Unclosed single quote".to_string()));
                }

                continue;
            }

            if c.is_whitespace() {
                break;
            }
            if matches!(c, '|' | ';' | '>' | '<' | '&') {
                break;
            }

            word.push(c);
            chars.next();
        }

        Ok(Token::Word(word))
    }
}
