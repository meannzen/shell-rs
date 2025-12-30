use crate::error::ShellError;
use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    Pipe,
    Semicolon,
    RedirectOut,
    RedirectIn,
    Background,
}

impl Token {
    pub fn tokenize(input: &str) -> Result<Vec<Token>, ShellError> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
                continue;
            }

            if c.is_ascii_digit() {
                let mut lookahead = chars.clone();
                lookahead.next(); // consume the digit
                if let Some(&next_c) = lookahead.peek() {
                    if matches!(next_c, '>' | '<') {
                        chars.next();
                        let redir_char = chars.next().unwrap();
                        tokens.push(if redir_char == '>' {
                            Token::RedirectOut
                        } else {
                            Token::RedirectIn
                        });
                        continue;
                    }
                }
            }

            match c {
                '|' => {
                    chars.next();
                    tokens.push(Token::Pipe);
                }
                ';' => {
                    chars.next();
                    tokens.push(Token::Semicolon);
                }
                '>' => {
                    chars.next();
                    tokens.push(Token::RedirectOut);
                }
                '<' => {
                    chars.next();
                    tokens.push(Token::RedirectIn);
                }
                '&' => {
                    chars.next();
                    tokens.push(Token::Background);
                }
                _ => {
                    let word = Token::read_word(&mut chars)?;
                    if !word.is_empty() {
                        tokens.push(Token::Word(word));
                    }
                }
            }
        }

        Ok(tokens)
    }

    fn read_word(chars: &mut Peekable<Chars>) -> Result<String, ShellError> {
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

            if c == '"' {
                chars.next();

                let mut found_closing = false;
                while let Some(&ch) = chars.peek() {
                    if ch == '"' {
                        chars.next();
                        found_closing = true;
                        break;
                    }
                    if ch == '\\' {
                        chars.next();
                        if let Some(&escaped) = chars.peek() {
                            match escaped {
                                '$' | '`' | '"' | '\\' => {
                                    word.push(escaped);
                                    chars.next();
                                }
                                '\n' => {
                                    chars.next();
                                }
                                _ => {
                                    word.push('\\');
                                    word.push(escaped);
                                    chars.next();
                                }
                            }
                        }
                    } else {
                        word.push(ch);
                        chars.next();
                    }
                }

                if !found_closing {
                    return Err(ShellError::ParseError("Unclosed double quote".to_string()));
                }

                continue;
            }

            if c.is_whitespace() {
                break;
            }
            if matches!(c, '|' | ';' | '>' | '<' | '&') {
                break;
            }

            if c == '\\' {
                chars.next();
                if let Some(&escaped) = chars.peek() {
                    word.push(escaped);
                    chars.next();
                }
            } else {
                word.push(c);
                chars.next();
            }
        }

        Ok(word)
    }
}

#[cfg(test)]
mod tests {
    use super::Token;
    #[test]
    fn test_backslash_escapes_spaces() {
        let input = r"echo world\ \ \ \ \ \ script";
        let tokens = Token::tokenize(input).unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Word("echo".to_string()));
        assert_eq!(tokens[1], Token::Word("world      script".to_string()));
    }

    #[test]
    fn test_backslash_with_mixed_spaces() {
        let input = r"echo before\ after";
        let tokens = Token::tokenize(input).unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Word("echo".to_string()));
        assert_eq!(tokens[1], Token::Word("before after".to_string()));
    }

    #[test]
    fn test_backslash_n_literal() {
        let input = r"echo test\nexample";
        let tokens = Token::tokenize(input).unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Word("echo".to_string()));
        assert_eq!(tokens[1], Token::Word("testnexample".to_string()));
    }

    #[test]
    fn test_escaped_backslash() {
        let input = r"echo hello\\world";
        let tokens = Token::tokenize(input).unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Word("echo".to_string()));
        assert_eq!(tokens[1], Token::Word("hello\\world".to_string()));
    }

    #[test]
    fn test_escaped_single_quotes() {
        let input = r"echo \'hello\'";
        let tokens = Token::tokenize(input).unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Word("echo".to_string()));
        assert_eq!(tokens[1], Token::Word("'hello'".to_string()));
    }
}
