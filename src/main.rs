use codecrafters_shell::{error::ShellError, shell::Shell};
use rustyline::Config;

fn main() -> Result<(), ShellError> {
    let config = Config::builder()
        .completion_type(rustyline::CompletionType::List)
        .build();

    let mut shell = Shell::new(config);
    shell.run();

    Ok(())
}

