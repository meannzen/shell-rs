use crate::{
    error::ShellError,
    executor::execute_pipeline,
    parser::{ast::Pipeline, lexer::Token, parse_tokens},
};
use std::{
    collections::HashMap,
    io::{BufRead, Write},
};

#[derive(Debug, Default)]
pub struct Shell {
    pub environment_var: HashMap<String, String>,
}

impl Shell {
    pub fn new() -> Self {
        Shell {
            environment_var: HashMap::new(),
        }
    }

    pub fn execute_pipelines(&mut self, pipelines: Vec<Pipeline>) {
        for pipeline in pipelines {
            match execute_pipeline(self, pipeline) {
                Ok(_exit_code) => {}
                Err(e) => match e {
                    ShellError::CommandNotFound(cmd) => {
                        eprintln!("{}: command not found", cmd);
                    }
                    _ => {
                        eprintln!("{}", e);
                    }
                },
            }
        }
    }

    fn parse_input(&mut self, input: &str) -> Result<Vec<Pipeline>, ShellError> {
        let tokens = Token::tokenize(input)?;
        let pipelines = parse_tokens(tokens)?;
        Ok(pipelines)
    }

    pub fn run(&mut self) {
        let mut reader = std::io::stdin().lock();
        let mut line = String::new();

        loop {
            print!("$ ");
            let _ = std::io::stdout().flush();

            line.clear();

            match reader.read_line(&mut line) {
                Ok(0) => {
                    println!("Exiting shell...");
                    break;
                }
                Ok(_) => {
                    let input = line.trim();
                    if input.is_empty() {
                        continue;
                    }

                    match self.parse_input(input) {
                        Ok(pipelines) => {
                            self.execute_pipelines(pipelines);
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Shell read error: {}", e);
                }
            }
        }
    }
}
