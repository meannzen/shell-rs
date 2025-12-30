use crate::{
    error::ShellError,
    parser::{
        ast::{Command, Pipeline},
        lexer::Token,
    },
};

pub mod ast;
pub mod lexer;

pub fn parse_tokens(tokens: Vec<Token>) -> Result<Vec<Pipeline>, ShellError> {
    let mut tokens_iter = tokens.into_iter().peekable();

    let mut pipelines: Vec<Pipeline> = Vec::new();

    while tokens_iter.peek().is_some() {
        let pipeline = parse_pipeline(&mut tokens_iter)?;
        pipelines.push(pipeline);
    }

    Ok(pipelines)
}
fn parse_pipeline(
    tokens_iter: &mut std::iter::Peekable<std::vec::IntoIter<Token>>,
) -> Result<Pipeline, ShellError> {
    let mut commands: Vec<Command> = Vec::new();
    commands.push(parse_command(tokens_iter)?);

    while let Some(token) = tokens_iter.peek() {
        if matches!(token, Token::Pipe) {
            tokens_iter.next();
            commands.push(parse_command(tokens_iter)?);
        } else {
            break;
        }
    }

    Ok(Pipeline { commands })
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
    let mut output_file: Option<String> = None;

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
            Token::RedirectOut => {
                tokens_iter.next();
                if let Some(Token::Word(file)) = tokens_iter.next() {
                    output_file = Some(file);
                } else {
                    return Err(ShellError::ParseError(
                        "Expected file name after '>'".to_string(),
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
        output: output_file,
    })
}
