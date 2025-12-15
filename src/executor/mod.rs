pub mod builtins;

use crate::{
    error::ShellError,
    executor::builtins::{execute_builtin, is_builtin},
    parser::ast::Pipeline,
    shell::Shell,
};

pub fn execute_pipeline(shell: &mut Shell, pipeline: Pipeline) -> Result<i32, ShellError> {
    let command = &pipeline.commands[0];
    if pipeline.commands.len() == 1 && is_builtin(&command.program) {
        return execute_builtin(shell, command);
    }

    Err(ShellError::CommandNotFound(format!(
        "{}: command not found",
        command.program
    )))
}
