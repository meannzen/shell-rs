use rustyline::{
    Cmd, Config, Editor, KeyEvent, completion::FilenameCompleter, config::Configurer,
    history::DefaultHistory,
};

#[cfg(unix)]
use std::path::Path;
use std::{
    collections::HashSet,
    env,
    sync::{Arc, Mutex},
};

use crate::{
    completer::MyHelper,
    error::ShellError,
    executor::execute_pipeline,
    parser::{ast::Pipeline, expand_variables, lexer::Token, parse_tokens},
    util,
};
use std::{
    collections::HashMap,
    fs::{self},
};

pub type VariableType = Arc<Mutex<HashMap<String, String>>>;

#[derive(Debug)]
pub struct Job {
    pub id: usize,
    pub pid: u32,
    pub cmd: String,
    pub child: std::process::Child,
    pub arguments: Vec<String>,
}

#[derive(Default)]
pub struct Shell {
    pub environment_var: HashMap<String, String>,
    config: Config,
    pub command_names: Vec<String>,
    pub histories: Vec<String>,
    pub history_append_index: usize,
    pub file_history_path: Option<String>,
    pub completions_regitry: Arc<Mutex<HashMap<String, String>>>,
    pub variables: VariableType,
    pub jobs: Vec<Job>,
}

impl Shell {
    pub fn build(config: Config, file_history_path: Option<String>) -> Self {
        let histories = file_history_path
            .as_deref()
            .map(util::read_history)
            .unwrap_or_default();

        let history_append_index = histories.len();
        let mut shell = Shell {
            environment_var: HashMap::new(),
            config,
            command_names: Vec::new(),
            histories,
            history_append_index,
            file_history_path,
            completions_regitry: Arc::new(Mutex::new(HashMap::new())),
            variables: Arc::new(Mutex::new(HashMap::new())),
            jobs: Vec::new(),
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

    pub fn reap_jobs(&mut self) {
        self.jobs.retain_mut(|job| {
            !matches!(job.child.try_wait(), Ok(Some(_)))
        });
    }

    fn parse_input(&mut self, input: &str) -> Result<Vec<Pipeline>, ShellError> {
        let tokens = Token::tokenize(input)?
            .into_iter()
            .filter_map(|t| match t {
                Token::Word(w) => {
                    let expanded = expand_variables(&w, &self.variables);
                    if expanded.is_empty() {
                        None
                    } else {
                        Some(Token::Word(expanded))
                    }
                }
                other => Some(other),
            })
            .collect();
        parse_tokens(tokens)
    }

    pub fn run(&mut self) {
        let mut rl: Editor<MyHelper, DefaultHistory> =
            Editor::with_config(self.config.clone()).unwrap();
        let registry = self.completions_regitry.clone();
        let h = MyHelper {
            file_completer: FilenameCompleter::new(),
            commands: self.command_names.clone(),
            registry,
        };

        rl.set_helper(Some(h));

        rl.bind_sequence(KeyEvent::from('\t'), Cmd::Complete);
        rl.set_auto_add_history(true);

        loop {
            self.reap_jobs();
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
