use rustyline::{
    Cmd, Config, Editor, KeyEvent, completion::FilenameCompleter, config::Configurer,
    history::DefaultHistory,
};

#[cfg(unix)]
use std::path::Path;
use std::{
    collections::HashSet,
    env,
    io::{BufRead, BufReader},
};

use crate::{
    completer::MyHelper,
    error::ShellError,
    executor::execute_pipeline,
    parser::{ast::Pipeline, lexer::Token, parse_tokens},
};
use std::{
    collections::HashMap,
    fs::{self},
};

#[derive(Debug, Default)]
pub struct Shell {
    pub environment_var: HashMap<String, String>,
    config: Config,
    pub command_names: Vec<String>,
    pub histories: Vec<String>,
    pub history_append_index: usize,
    pub file_history_path: Option<String>,
}

impl Shell {
    pub fn build(config: Config, file_history_path: Option<String>) -> Self {
        let mut histories = Vec::new();
        if let Some(path) = &file_history_path
            && let Ok(file) = std::fs::File::open(path)
        {
            let reader = BufReader::new(file);
            for line in reader.lines().map_while(|x| x.ok()) {
                histories.push(line);
            }
        }

        let mut shell = Shell {
            environment_var: HashMap::new(),
            config,
            command_names: Vec::new(),
            histories,
            history_append_index: 0,
            file_history_path,
        };

        shell.command_names = shell.collect_command_names();
        shell
    }

    fn collect_command_names(&self) -> Vec<String> {
        let mut names = HashSet::new();

        for builtin in ["echo", "exit"] {
            names.insert(builtin.to_string());
        }

        if let Ok(path_str) = env::var("PATH") {
            for dir_path in env::split_paths(&path_str) {
                if !dir_path.as_os_str().is_empty()
                    && let Ok(entries) = fs::read_dir(&dir_path)
                {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file()
                            && Self::is_executable(&path)
                            && let Some(file_name) = path.file_name().and_then(|n| n.to_str())
                        {
                            names.insert(file_name.to_string());
                        }
                    }
                }
            }
        }

        let mut vec: Vec<_> = names.into_iter().collect();
        vec.sort();
        vec.dedup();
        vec
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

    #[cfg(unix)]
    fn is_executable(path: &Path) -> bool {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
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
            commands: self.command_names.clone(),
        };

        rl.set_helper(Some(h));

        rl.bind_sequence(KeyEvent::from('\t'), Cmd::Complete);
        rl.set_auto_add_history(true);

        loop {
            let readline = rl.readline("$ ");
            match readline {
                Ok(line) => {
                    let input = line.trim();
                    if input.is_empty() {
                        continue;
                    }

                    self.histories.push(line.clone());

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
