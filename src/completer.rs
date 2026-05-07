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
    ) -> std::result::Result<(usize, Vec<Pair>), rustyline::error::ReadlineError> {
        let sub_line = &line[..pos];

        let words: Vec<&str> = sub_line.split_ascii_whitespace().collect();
        if words.is_empty() {
            return Ok((0, Vec::new()));
        }

        let cmd_name = words[0];
        let registry = self.registry.lock().unwrap();

        if let Some(script_path) = registry.get(cmd_name) {
            let (current_word, start_pos) = if sub_line.ends_with(' ') || sub_line.is_empty() {
                ("", pos)
            } else {
                let last_word = words.last().unwrap_or(&"");
                let last_word_start = sub_line.rfind(last_word).unwrap_or(pos);
                (*last_word, last_word_start)
            };

            let prev_word = if sub_line.ends_with(' ') {
                words.last().unwrap_or(&"")
            } else {
                if words.len() > 1 {
                    words[words.len() - 2]
                } else {
                    ""
                }
            };

            let output = Command::new(script_path)
                .arg(cmd_name)
                .arg(current_word)
                .arg(prev_word)
                .env("COMP_LINE", line)
                .env("COMP_POINT", pos.to_string())
                .output();

            if let Ok(out) = output {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if let Some(candidate) = stdout.lines().next() {
                    let trimmed = candidate.trim();
                    if trimmed.is_empty() {
                        return Ok((0, Vec::new()));
                    }

                    let pair = Pair {
                        display: trimmed.to_string(),
                        replacement: format!("{} ", trimmed),
                    };

                    return Ok((start_pos, vec![pair]));
                }
            }
        }

        // Fall back to command name completion when completing the first word
        if words.len() == 1 && !sub_line.ends_with(' ') {
            let matches: Vec<Pair> = self
                .commands
                .iter()
                .filter(|c| c.starts_with(cmd_name))
                .map(|c| Pair {
                    display: c.clone(),
                    replacement: format!("{} ", c),
                })
                .collect();
            return Ok((0, matches));
        }

        Ok((0, Vec::new()))
    }
}

impl rustyline::hint::Hinter for MyHelper {
    type Hint = String;
}

impl rustyline::highlight::Highlighter for MyHelper {}
impl rustyline::validate::Validator for MyHelper {}
impl Helper for MyHelper {}
