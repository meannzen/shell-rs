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
    let command = parse_command(tokens_iter)?;
    commands.push(command);

    Ok(Pipeline { commands })
}

fn parse_command(
    tokens_iter: &mut std::iter::Peekable<std::vec::IntoIter<Token>>,
) -> Result<Command, ShellError> {
    let program = match tokens_iter.next() {
        Some(Token::Word(s)) => s,
        None => {
            return Err(ShellError::ParseError(
                "Unexpected end of input".to_string(),
            ));
        }
    };

    let mut arguments: Vec<String> = Vec::new();
    let input_file: Option<String> = None;
    let output_mode: Option<String> = None;

    while tokens_iter.peek().is_some() {
        match tokens_iter.peek() {
            Some(Token::Word(arg)) => {
                arguments.push(arg.to_string());
            }
            _ => {
                break;
            }
        }
    }

    Ok(Command {
        program,
        arguments,
        input: input_file,
        output: output_mode,
    })
}
