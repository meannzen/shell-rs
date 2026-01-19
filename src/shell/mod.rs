use rustyline::{
    Cmd, Config, Editor, KeyEvent, completion::FilenameCompleter, history::DefaultHistory,
};

use crate::{
    completer::MyHelper,
    error::ShellError,
    executor::execute_pipeline,
    parser::{ast::Pipeline, lexer::Token, parse_tokens},
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Shell {
    pub environment_var: HashMap<String, String>,
    config: Config,
}

impl Shell {
    pub fn new(config: Config) -> Self {
        Shell {
            environment_var: HashMap::new(),
            config,
        }
    }

    pub fn execute_pipelines(&mut self, pipelines: Vec<Pipeline>) {
        for pipeline in pipelines {
            match execute_pipeline(self, pipeline) {
                Ok(_exit_code) => {}
                Err(e) => match e {
                    ShellError::CommandNotFound(cmd) => {
                        eprintln!("{}", cmd);
                    }
                    ShellError::InternalError(msg) => {
                        eprintln!("{}", msg);
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
        let mut rl: Editor<MyHelper, DefaultHistory> =
            Editor::with_config(self.config.clone()).unwrap();

        let h = MyHelper {
            file_completer: FilenameCompleter::new(),
            commands: vec!["echo ".to_string(), "exit ".to_string()],
        };

        rl.set_helper(Some(h));

        rl.bind_sequence(KeyEvent::from('\t'), Cmd::Complete);

        loop {
            let readline = rl.readline("$ ");
            match readline {
                Ok(line) => {
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
