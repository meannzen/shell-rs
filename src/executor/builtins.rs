use std::env;

use crate::{error::ShellError, parser::ast::Command, shell::Shell};

const BUILTINS: &[&str] = &["exit", "echo", "type", "pwd", "cd"];

pub fn is_builtin(program: &str) -> bool {
    BUILTINS.contains(&program)
}

pub fn execute_builtin(_shell: &mut Shell, command: &Command) -> Result<i32, ShellError> {
    match command.program.as_str() {
        "exit" => execute_exit(&command.arguments),
        "echo" => execute_echo(&command.arguments),
        "type" => execute_type(&command.arguments),
        "pwd" => execute_pwd(),
        "cd" => execute_cd(&command.arguments),
        _ => Err(ShellError::CommandNotFound(format!(
            "{}: command not found",
            command.program
        ))),
    }
}

fn execute_pwd() -> Result<i32, ShellError> {
    let current_path = env::current_dir()?;
    println!("{}", current_path.display());
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

fn execute_echo(args: &[String]) -> Result<i32, ShellError> {
    if args.is_empty() {
        println!();
        return Ok(0);
    }

    let output = args.join(" ");
    println!("{}", output);

    Ok(0)
}
fn execute_type(args: &[String]) -> Result<i32, ShellError> {
    if args.is_empty() {
        return Err(ShellError::InternalError(
            "need at least one argument".to_string(),
        ));
    }

    let program = &args[0];
    if is_builtin(program) {
        println!("{} is a shell builtin", program);
        return Ok(0);
    }

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

    let output = str::from_utf8(&output.stdout)
        .map_err(|_| ShellError::InternalError("Not valid UTF-8".to_string()))?;

    print!("{} is {}", program, output);
    Ok(0)
}
