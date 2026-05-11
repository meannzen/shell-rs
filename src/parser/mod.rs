use crate::{
    error::ShellError,
    parser::{
        ast::{Command, Pipeline, Redirection},
        lexer::Token,
    },
    shell::VariableType,
};

pub fn expand_variables(s: &str, variables: &VariableType) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' {
            let var_name: String = if chars.peek() == Some(&'{') {
                chars.next();
                let name: String = chars.by_ref().take_while(|c| *c != '}').collect();
                name
            } else {
                chars
                    .by_ref()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect()
            };
            if var_name.is_empty() {
                result.push('$');
            } else {
                if let Some(value) = variables.lock().unwrap().get(&var_name).cloned() {
                    result.push_str(&value);
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

pub mod ast;
pub mod lexer;

pub fn parse_tokens(tokens: Vec<Token>) -> Result<Vec<Pipeline>, ShellError> {
    let mut tokens_iter = tokens.into_iter().peekable();
    let mut pipelines: Vec<Pipeline> = Vec::new();
    while tokens_iter.peek().is_some() {
        pipelines.push(parse_pipeline(&mut tokens_iter)?);
    }
    Ok(pipelines)
}

fn parse_pipeline(
    tokens_iter: &mut std::iter::Peekable<std::vec::IntoIter<Token>>,
) -> Result<Pipeline, ShellError> {
    let mut commands: Vec<Command> = Vec::new();
    let mut background = false;
    commands.push(parse_command(tokens_iter)?);
    while let Some(token) = tokens_iter.peek() {
        if matches!(token, Token::Pipe) {
            tokens_iter.next();
            commands.push(parse_command(tokens_iter)?);
        } else if matches!(token, Token::Background) {
            tokens_iter.next();
            background = true;
            break;
        } else {
            break;
        }
    }
    Ok(Pipeline { commands, background })
}

fn parse_command(
    tokens_iter: &mut std::iter::Peekable<std::vec::IntoIter<Token>>,
) -> Result<Command, ShellError> {
    let program = match tokens_iter.next() {
        Some(Token::Word(s)) => s,
        _ => {
            return Err(ShellError::ParseError(
                "Unexpected end of input".to_string(),
            ));
        }
    };

    let mut arguments: Vec<String> = Vec::new();
    let mut input_file: Option<String> = None;
    let mut output_redirections: Vec<Redirection> = Vec::new();

    while let Some(token) = tokens_iter.peek() {
        match token {
            Token::Pipe | Token::Semicolon | Token::Background => break,
            Token::RedirectIn => {
                tokens_iter.next();
                if let Some(Token::Word(file)) = tokens_iter.next() {
                    input_file = Some(file);
                } else {
                    return Err(ShellError::ParseError(
                        "Expected file name after '<'".to_string(),
                    ));
                }
            }
            Token::RedirectOut(fd) => {
                let fd = *fd;
                tokens_iter.next();
                if let Some(Token::Word(file)) = tokens_iter.next() {
                    output_redirections.push(Redirection {
                        path: file,
                        fd,
                        append: false,
                    });
                } else {
                    return Err(ShellError::ParseError(
                        "Expected file name after '>'".to_string(),
                    ));
                }
            }
            Token::RedirectAppend(fd) => {
                let fd = *fd;
                tokens_iter.next();
                if let Some(Token::Word(file)) = tokens_iter.next() {
                    output_redirections.push(Redirection {
                        path: file,
                        fd,
                        append: true,
                    });
                } else {
                    return Err(ShellError::ParseError(
                        "Expected file name after '>>'".to_string(),
                    ));
                }
            }
            Token::Word(arg) => {
                arguments.push(arg.clone());
                tokens_iter.next();
            }
        }
    }

    Ok(Command {
        program,
        arguments,
        input: input_file,
        outputs: output_redirections,
    })
}
