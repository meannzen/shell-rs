use std::path::Path;
use std::process::Command;

use crate::{error::ShellError, parser::ast::Pipeline, shell::Shell};

fn find_executable_in_path(command: &str) -> bool {
    if let Ok(path_var) = std::env::var("PATH") {
        for path in path_var.split(':') {
            let full_path = Path::new(path).join(command);
            if full_path.is_file() {
                return true;
            }
        }
    }
    false
}

pub fn execute_pipeline(_shell: &mut Shell, pipeline: Pipeline) -> Result<i32, ShellError> {
    for command in pipeline.commands {
        if !find_executable_in_path(&command.program) {
            return Err(ShellError::CommandNotFound(format!(
                "{}: command not found",
                command.program
            )));
        }

        let mut cmd = Command::new(&command.program);
        cmd.args(&command.arguments);

        match cmd.status() {
            Ok(status) => {
                if !status.success() {
                    // The command executed, but returned a non-zero exit code.
                    // We don't need to do anything here, as the command's output/error
                    // will have been piped to the shell's stdout/stderr.
                }
            }
            Err(_) => {
                // This would happen if the command fails to spawn, e.g., due to permissions.
                // We've already checked if the command exists, so this is a different error.
                return Err(ShellError::CommandNotFound(format!(
                    "{}: command not found",
                    command.program
                )));
            }
        }
    }
    Ok(0)
}
