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

        let mut stdout = if i < pipeline.commands.len() - 1 {
            Stdio::piped()
        } else {
            Stdio::inherit()
        };
        let mut stderr = Stdio::inherit();

        if let Some(value) = &command.output {
            let file = std::fs::File::create(value.0.clone())?;
            if value.1 == 1 {
                stdout = Stdio::from(file);
            } else if value.1 == 2 {
                stderr = Stdio::from(file);
            }
        }

        let mut child = Command::new(&command.program)
            .args(&command.arguments)
            .stdin(stdin)
            .stdout(stdout)
            .stderr(stderr)
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
