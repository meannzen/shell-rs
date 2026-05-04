use std::env;

use codecrafters_shell::{error::ShellError, shell::Shell};
use rustyline::Config;

fn main() -> Result<(), ShellError> {
    let config = Config::builder()
        .completion_type(rustyline::CompletionType::List)
        .build();

    let file_path = env::var("HISTFILE").ok();

    Shell::build(config, file_path).run();

    Ok(())
}
