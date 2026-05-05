use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};

use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::{Context, Helper};

pub struct MyHelper {
    pub file_completer: FilenameCompleter,
    pub commands: Vec<String>,
    pub registry: Arc<Mutex<HashMap<String, String>>>,
}

impl Completer for MyHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if !parts.is_empty() && line.ends_with(' ') {
            let cmd_name = parts[0];
            let registry = self.registry.lock().unwrap();
            if let Some(script_path) = registry.get(cmd_name) {
                let output = Command::new(script_path).output();

                if let Ok(out) = output {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    if let Some(candidate) = stdout.lines().next() {
                        let trimmed_candidate = candidate.trim();
                        let replacement = format!("{} ", trimmed_candidate);

                        let pair = Pair {
                            display: trimmed_candidate.to_string(),
                            replacement: replacement.clone(),
                        };

                        return Ok((pos, vec![pair]));
                    }
                }
            }
        }

        let word_start = line[..pos]
            .rfind(char::is_whitespace)
            .map(|i| i + 1)
            .unwrap_or(0);

        let being_completed = &line[word_start..pos];

        if word_start == 0 && !being_completed.is_empty() {
            let mut candidates = Vec::new();

            for cmd in &self.commands {
                if cmd.starts_with(being_completed) {
                    let completion = if cmd.contains(' ') {
                        format!("'{}' ", cmd)
                    } else {
                        format!("{} ", cmd)
                    };

                    candidates.push(Pair {
                        display: cmd.clone(),
                        replacement: completion,
                    });
                }
            }

            if !candidates.is_empty() {
                return Ok((word_start, candidates));
            }
        }

        self.file_completer.complete(line, pos, _ctx)
    }
}
impl rustyline::hint::Hinter for MyHelper {
    type Hint = String;
}

impl rustyline::highlight::Highlighter for MyHelper {}
impl rustyline::validate::Validator for MyHelper {}
impl Helper for MyHelper {}
