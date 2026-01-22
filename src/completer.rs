use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::{Context, Helper};

pub struct MyHelper {
    pub file_completer: FilenameCompleter,
    pub commands: Vec<String>,
}

impl Completer for MyHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
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
