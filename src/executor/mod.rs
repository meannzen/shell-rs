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
    let num_commands = pipeline.commands.len();

    for (i, command) in pipeline.commands.iter().enumerate() {
        let is_first = i == 0;
        let is_last = i == num_commands - 1;

        let stdin = if is_first {
            if let Some(input_file) = &command.input {
                let file = std::fs::File::open(input_file)?;
                Stdio::from(file)
            } else {
                Stdio::inherit()
            }
        } else {
            previous_stdout.take().unwrap_or(Stdio::inherit())
        };

        let mut stdout = if !is_last {
            Stdio::piped()
        } else {
            Stdio::inherit()
        };
        let mut stderr = Stdio::inherit();

        for redir in &command.outputs {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .append(redir.append)
                .truncate(!redir.append)
                .open(&redir.path)?;

            if redir.fd == 1 {
                stdout = Stdio::from(file);
            } else if redir.fd == 2 {
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

        if let Some(stdout_pipe) = child.stdout.take() {
            previous_stdout = Some(Stdio::from(stdout_pipe));
        }

        children.push(child);
    }

    let mut last_status = 0;
    for (i, mut child) in children.into_iter().enumerate() {
        let status = child.wait()?;
        if i == num_commands - 1 {
            last_status = status.code().unwrap_or(0);
        }
    }

    Ok(last_status)
}
