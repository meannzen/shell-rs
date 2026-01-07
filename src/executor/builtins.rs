use std::{
    env,
    fs::OpenOptions,
    io::{self, Write},
    str,
};

use crate::{error::ShellError, parser::ast::Command, shell::Shell};

const BUILTINS: &[&str] = &["exit", "echo", "type", "pwd", "cd"];

pub fn is_builtin(program: &str) -> bool {
    BUILTINS.contains(&program)
}

pub fn execute_builtin(_shell: &mut Shell, command: &Command) -> Result<i32, ShellError> {
    match command.program.as_str() {
        "exit" => execute_exit(&command.arguments),
        "echo" => execute_echo(command),
        "type" => execute_type(command),
        "pwd" => execute_pwd(command),
        "cd" => execute_cd(&command.arguments),
        _ => Err(ShellError::CommandNotFound(format!(
            "{}: command not found",
            command.program
        ))),
    }
}

fn get_command_writer(command: &Command) -> Result<Box<dyn Write>, ShellError> {
    let mut writer: Box<dyn Write> = Box::new(io::stdout());
    for redir in &command.outputs {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(redir.append)
            .truncate(!redir.append)
            .open(&redir.path)?;

        if redir.fd == 1 {
            writer = Box::new(file);
        }
    }
    Ok(writer)
}

fn execute_pwd(command: &Command) -> Result<i32, ShellError> {
    let current_path = env::current_dir()?;
    let mut writer = get_command_writer(command)?;
    writeln!(writer, "{}", current_path.display())?;
    Ok(0)
}

fn execute_cd(args: &[String]) -> Result<i32, ShellError> {
    let home = std::env::home_dir();
    let new_dir = if args.is_empty() {
        home.clone()
            .ok_or_else(|| ShellError::InternalError("cd: no home directory found".to_string()))?
    } else if args.len() != 1 {
        return Err(ShellError::InternalError(
            "cd: too many arguments".to_string(),
        ));
    } else {
        let raw = &args[0];
        if raw == "~" {
            home.clone().ok_or_else(|| {
                ShellError::InternalError("cd: no home directory found".to_string())
            })?
        } else if let Some(stripped) = raw.strip_prefix("~/") {
            home.clone()
                .ok_or_else(|| {
                    ShellError::InternalError("cd: no home directory found".to_string())
                })?
                .join(stripped)
        } else {
            std::path::PathBuf::from(raw)
        }
    };

    if std::env::set_current_dir(&new_dir).is_ok() {
        Ok(0)
    } else {
        let display = if Some(&new_dir) == home.as_ref() {
            "~".to_string()
        } else if let Some(h) = home {
            if let Ok(rel) = new_dir.strip_prefix(&h) {
                format!("~{}", rel.display())
            } else {
                new_dir.display().to_string()
            }
        } else {
            new_dir.display().to_string()
        };

        Err(ShellError::InternalError(format!(
            "cd: {}: No such file or directory",
            display
        )))
    }
}

fn execute_exit(args: &[String]) -> Result<i32, ShellError> {
    let exit_code = args
        .first()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);

    std::process::exit(exit_code);
}

fn execute_echo(command: &Command) -> Result<i32, ShellError> {
    let mut writer = get_command_writer(command)?;
    let output = command.arguments.join(" ");
    writeln!(writer, "{}", output)?;
    Ok(0)
}

fn execute_type(command: &Command) -> Result<i32, ShellError> {
    let args = &command.arguments;
    if args.is_empty() {
        return Err(ShellError::InternalError(
            "need at least one argument".to_string(),
        ));
    }

    let program = &args[0];
    let mut writer = get_command_writer(command)?;

    if is_builtin(program) {
        writeln!(writer, "{} is a shell builtin", program)?;
    } else {
        let output = std::process::Command::new("which")
            .arg(program)
            .output()
            .map_err(|_| ShellError::CommandNotFound(format!("{}: not found", program)))?;

        if !output.status.success() || output.stdout.is_empty() {
            return Err(ShellError::CommandNotFound(format!(
                "{}: not found",
                program
            )));
        }

        let stdout_str = str::from_utf8(&output.stdout)
            .map_err(|_| ShellError::InternalError("Not valid UTF-8".to_string()))?;

        write!(writer, "{} is {}", program, stdout_str)?;
        if !stdout_str.ends_with('\n') {
            writeln!(writer)?;
        }
    };

    Ok(0)
}
