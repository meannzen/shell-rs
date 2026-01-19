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
        ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let mut entries: Vec<Pair> = self
            .commands
            .iter()
            .filter(|cmd| cmd.starts_with(line))
            .map(|cmd| Pair {
                display: cmd.to_string(),
                replacement: cmd.to_string(),
            })
            .collect();

        let (_, file_entries) = self.file_completer.complete(line, pos, ctx)?;
        entries.extend(file_entries);

        Ok((0, entries))
    }
}

impl rustyline::hint::Hinter for MyHelper {
    type Hint = String;
}

impl rustyline::highlight::Highlighter for MyHelper {}
impl rustyline::validate::Validator for MyHelper {}
impl Helper for MyHelper {}
