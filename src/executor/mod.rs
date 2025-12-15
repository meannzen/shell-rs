pub mod builtins;

use crate::{
    error::ShellError,
    executor::builtins::{execute_builtin, is_builtin},
    parser::ast::Pipeline,
    shell::Shell,
};
use std::process::{Command, Stdio};

pub fn execute_pipeline(shell: &mut Shell, pipeline: Pipeline) -> Result<i32, ShellError> {
    if pipeline.commands.is_empty() {
        return Ok(0);
    }

    if pipeline.commands.len() == 1 {
        let command = &pipeline.commands[0];
        if is_builtin(&command.program) {
            return execute_builtin(shell, command);
        }
    }

    let mut previous_stdout: Option<Stdio> = None;
    let mut children = vec![];

    for (i, command) in pipeline.commands.iter().enumerate() {
        let stdin = if i == 0 {
            if let Some(input_file) = &command.input {
                let file = std::fs::File::open(input_file)?;
                Stdio::from(file)
            } else {
                Stdio::inherit()
            }
        } else {
            previous_stdout.take().unwrap_or(Stdio::inherit())
        };

        let stdout = if i < pipeline.commands.len() - 1 {
            Stdio::piped()
        } else if let Some(output_file) = &command.output {
            let file = std::fs::File::create(output_file)?;
            Stdio::from(file)
        } else {
            Stdio::inherit()
        };

        let mut child = Command::new(&command.program)
            .args(&command.arguments)
            .stdin(stdin)
            .stdout(stdout)
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    ShellError::CommandNotFound(format!("{}: command not found", command.program))
                } else {
                    e.into()
                }
            })?;

        if let Some(stdout) = child.stdout.take() {
            previous_stdout = Some(Stdio::from(stdout));
        }

        children.push(child);
    }

    for mut child in children {
        child.wait()?;
    }

    Ok(0)
}
