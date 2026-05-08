use std::{
    env,
    fs::OpenOptions,
    io::{self, Write},
    str,
};

use crate::{error::ShellError, parser::ast::Command, shell::Shell, util};

const BUILTINS: &[&str] = &[
    "exit", "echo", "type", "pwd", "cd", "history", "complete", "declare",
];

pub fn is_builtin(program: &str) -> bool {
    BUILTINS.contains(&program)
}

pub fn execute_builtin(
    shell: &mut Shell,
    command: &Command,
    stdout_override: Option<Box<dyn Write>>,
) -> Result<i32, ShellError> {
    match command.program.as_str() {
        "exit" => execute_exit(shell, &command.arguments),
        "echo" => execute_echo(command, stdout_override),
        "type" => execute_type(command, stdout_override),
        "pwd" => execute_pwd(command, stdout_override),
        "cd" => execute_cd(&command.arguments),
        "history" => execute_history(shell, command, stdout_override),
        "complete" => execute_complete(shell, command, stdout_override),
        "declare" => execute_declare(shell, command, stdout_override),
        _ => Err(ShellError::CommandNotFound(format!(
            "{}: command not found",
            command.program
        ))),
    }
}

fn execute_declare(
    shell: &mut Shell,
    command: &Command,
    stdout_override: Option<Box<dyn Write>>,
) -> Result<i32, ShellError> {
    let mut writer = get_command_writer(command, stdout_override)?;

    if let Some(first_arg) = command.arguments.first() {
        match first_arg.as_str() {
            "-p" => {
                let key = match command.arguments.get(1) {
                    Some(v) => v,
                    None => "",
                };

                if let Some(value) = shell.variables.get(key) {
                    writeln!(writer, "declare -- {}=\"{}\"", key, value)?;
                    return Ok(0);
                }
                writeln!(writer, "declare: {}: not found", key)?;
            }
            str_valus => {
                let arrays: Vec<_> = str_valus.splitn(2, "=").collect();
                if let Some(key) = arrays.first()
                    && let Some(value) = arrays.get(1)
                {
                    let trimmed = key.trim_ascii();
                    if trimmed.starts_with(|c: char| c.is_ascii_digit())
                        || trimmed.contains(|c: char| c.is_ascii_punctuation() && c != '_')
                    {
                        writeln!(
                            writer,
                            "declare: `{}={}': not a valid identifier",
                            key, value
                        )?;
                        return Ok(1);
                    }
                    shell.variables.insert(key.to_string(), value.to_string());
                    return Ok(0);
                }
            }
        }
    }
    Ok(1)
}

fn execute_complete(
    shell: &mut Shell,
    command: &Command,
    stdout_override: Option<Box<dyn Write>>,
) -> Result<i32, ShellError> {
    let mut writer = get_command_writer(command, stdout_override)?;
    if let Some(first) = command.arguments.first() {
        match first.as_str() {
            "-p" => {
                if let Some(cmd_name) = command.arguments.get(1) {
                    let completions_regitry = shell.completions_regitry.lock().unwrap();
                    if let Some(path) = completions_regitry.get(cmd_name) {
                        writeln!(writer, "complete -C '{}' {}", path, cmd_name)?;
                    } else {
                        writeln!(
                            writer,
                            "complete: {}: no completion specification",
                            cmd_name
                        )?;
                    }
                }
            }
            "-r" => {
                if let Some(cmd_name) = command.arguments.get(1) {
                    let mut completions_regitry = shell.completions_regitry.lock().unwrap();
                    completions_regitry.remove(cmd_name);
                }
            }
            "-C" => {
                if let Some(two) = command.arguments.get(1)
                    && let Some(third) = command.arguments.get(2)
                {
                    let mut completions_regitry = shell.completions_regitry.lock().unwrap();
                    completions_regitry.insert(third.clone(), two.clone());
                }
            }
            _ => {
                todo!()
            }
        }
    }
    Ok(1)
}

fn get_command_writer(
    command: &Command,
    stdout_override: Option<Box<dyn Write>>,
) -> Result<Box<dyn Write>, ShellError> {
    let mut writer: Box<dyn Write> = stdout_override.unwrap_or_else(|| Box::new(io::stdout()));
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

fn execute_history(
    shell: &mut Shell,
    command: &Command,
    stdout_override: Option<Box<dyn Write>>,
) -> Result<i32, ShellError> {
    let mut writer = get_command_writer(command, stdout_override)?;
    let binding = String::from("0");
    let first_arg = command.arguments.first().unwrap_or(&binding).as_str();

    match first_arg {
        "-r" => {
            if let Some(path) = command.arguments.get(1) {
                let lines = util::read_history(path);
                shell.histories.extend(lines);
                shell.history_append_index = shell.histories.len();
            }
            return Ok(0);
        }
        "-w" => {
            if let Some(path) = command.arguments.get(1) {
                util::write_history(path, &shell.histories)?;
                return Ok(0);
            }
            return Ok(1);
        }
        "-a" => {
            if let Some(path) = command.arguments.get(1) {
                let new_index =
                    util::append_history(path, &shell.histories, shell.history_append_index)?;
                shell.history_append_index = new_index;
                return Ok(0);
            }
            return Ok(1);
        }
        _ => {}
    }

    let some_n = first_arg.parse();

    if let Ok(n) = some_n
        && n > 0
        && n < shell.histories.len()
    {
        let historis: Vec<_> = shell.histories.iter().rev().take(n).collect();
        for (i, history) in historis.iter().rev().enumerate() {
            writeln!(writer, "{} {}", (shell.histories.len() + i) - n, history)?;
        }
    } else {
        for (i, history) in shell.histories.iter().enumerate() {
            writeln!(writer, "{} {}", i + 1, history)?;
        }
    }
    Ok(0)
}

fn execute_pwd(
    command: &Command,
    stdout_override: Option<Box<dyn Write>>,
) -> Result<i32, ShellError> {
    let current_path = env::current_dir()?;
    let mut writer = get_command_writer(command, stdout_override)?;
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

fn execute_exit(shell: &Shell, args: &[String]) -> Result<i32, ShellError> {
    let exit_code = args
        .first()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);

    if let Some(path) = &shell.file_history_path {
        let _ = util::append_history(path, &shell.histories, shell.history_append_index);
    }

    std::process::exit(exit_code);
}

fn execute_echo(
    command: &Command,
    stdout_override: Option<Box<dyn Write>>,
) -> Result<i32, ShellError> {
    let mut writer = get_command_writer(command, stdout_override)?;
    let output = command.arguments.join(" ");
    writeln!(writer, "{}", output)?;
    Ok(0)
}

fn execute_type(
    command: &Command,
    stdout_override: Option<Box<dyn Write>>,
) -> Result<i32, ShellError> {
    let args = &command.arguments;
    if args.is_empty() {
        return Err(ShellError::InternalError(
            "need at least one argument".to_string(),
        ));
    }

    let program = &args[0];
    let mut writer = get_command_writer(command, stdout_override)?;

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
