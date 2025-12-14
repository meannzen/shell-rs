#[allow(unused_imports)]
use std::io::{self, Write};

use codecrafters_shell::{error::ShellError, shell::Shell};

fn main() -> Result<(), ShellError> {
    let mut shell = Shell::new();
    shell.run();

    Ok(())
}
