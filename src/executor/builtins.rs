use std::{env, fs::OpenOptions, io::Write};

use crate::{error::ShellError, parser::ast::Command, shell::Shell};

const BUILTINS: &[&str] = &["exit", "echo", "type", "pwd", "cd"];

pub fn is_builtin(program: &str) -> bool {
    BUILTINS.contains(&program)
}

pub fn execute_builtin(_shell: &mut Shell, command: &Command) -> Result<i32, ShellError> {
    match command.program.as_str() {
        "exit" => execute_exit(&command.arguments),
        "echo" => execute_echo(&command),
        "type" => execute_type(&command),
        "pwd" => execute_pwd(&command),
        "cd" => execute_cd(&command.arguments),
        _ => Err(ShellError::CommandNotFound(format!(
            "{}: command not found",
            command.program
        ))),
    }
}

fn execute_pwd(command: &Command) -> Result<i32, ShellError> {
    let current_path = env::current_dir()?;
    let output = format!("{}", current_path.display());

    if let Some(value) = &command.output {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(value.0.clone())?;

        if value.1 == 1 {
            file.write_all(output.as_bytes())?;
            file.write_all(b"\n")?;
            return Ok(0);
        }
    }

    println!("{}", output);
    Ok(0)
}

fn execute_cd(args: &[String]) -> Result<i32, ShellError> {
    let new_dir = if args.is_empty() {
        if let Some(home) = std::env::home_dir() {
            home
        } else {
            return Err(ShellError::InternalError(
                "cd: no home directory found".to_string(),
            ));
        }
    } else if args.len() != 1 {
        return Err(ShellError::InternalError(
            "cd: too many arguments".to_string(),
        ));
    } else {
        let raw = &args[0];

        if raw == "~" {
            if let Some(home) = std::env::home_dir() {
                home
            } else {
                return Err(ShellError::InternalError(
                    "cd: no home directory found".to_string(),
                ));
            }
        } else if let Some(stripped) = raw.strip_prefix("~/") {
            if let Some(home) = std::env::home_dir() {
                home.join(stripped)
            } else {
                return Err(ShellError::InternalError(
                    "cd: no home directory found".to_string(),
                ));
            }
        } else {
            std::path::PathBuf::from(raw)
        }
    };

    if std::env::set_current_dir(&new_dir).is_ok() {
        Ok(0)
    } else {
        let display = if new_dir == std::env::home_dir().unwrap_or_default() {
            "~".to_string()
        } else if let Some(home) = std::env::home_dir() {
            if let Ok(rel) = new_dir.strip_prefix(&home) {
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
    if command.arguments.is_empty() {
        println!();
        return Ok(0);
    }

    if let Some(value) = &command.output {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(value.0.clone())?;

        if value.1 == 1 {
            let buff = command.arguments.join(" ");
            file.write_all(buff.as_bytes())?;
            file.write_all(b"\n")?;
            return Ok(0);
        }
    }

    let output = command.arguments.join(" ");
    println!("{}", output);

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
    let output_msg = if is_builtin(program) {
        format!("{} is a shell builtin", program)
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

        format!("{} is {}", program, stdout_str)
    };

    if let Some(value) = &command.output {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(value.0.clone())?;

        if value.1 == 1 {
            file.write_all(output_msg.as_bytes())?;
            // only add newline if it doesn't end with one (which is mostly for the external case)
            if !output_msg.ends_with('\n') {
                file.write_all(b"\n")?;
            }
            return Ok(0);
        }
    }

    if output_msg.ends_with('\n') {
        print!("{}", output_msg);
    } else {
        println!("{}", output_msg);
    }
    Ok(0)
}
