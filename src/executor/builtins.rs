use std::{
    env,
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, Read, Seek, Write},
    str,
};

use crate::{error::ShellError, parser::ast::Command, shell::Shell};

const BUILTINS: &[&str] = &["exit", "echo", "type", "pwd", "cd", "history"];

pub fn is_builtin(program: &str) -> bool {
    BUILTINS.contains(&program)
}

pub fn execute_builtin(
    shell: &mut Shell,
    command: &Command,
    stdout_override: Option<Box<dyn Write>>,
) -> Result<i32, ShellError> {
    match command.program.as_str() {
        "exit" => execute_exit(&command.arguments),
        "echo" => execute_echo(command, stdout_override),
        "type" => execute_type(command, stdout_override),
        "pwd" => execute_pwd(command, stdout_override),
        "cd" => execute_cd(&command.arguments),
        "history" => execute_history(shell, command, stdout_override),
        _ => Err(ShellError::CommandNotFound(format!(
            "{}: command not found",
            command.program
        ))),
    }
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
                let file = File::open(path)?;
                let reader = BufReader::new(file);
                for line in reader.lines().map_while(Result::ok) {
                    shell.histories.push(line);
                }
                shell.history_append_index = shell.histories.len();
            }
            return Ok(0);
        }
        "-w" => {
            if let Some(path) = command.arguments.get(1) {
                let mut file = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(path)?;

                for line in &shell.histories {
                    writeln!(file, "{}", line)?;
                }

                return Ok(0);
            }
            return Ok(1);
        }
        "-a" => {
            if let Some(path) = command.arguments.get(1) {
                let write_start = match File::open(path) {
                    Ok(mut f) => {
                        let file_len = f.seek(io::SeekFrom::End(0)).unwrap_or(0);
                        if file_len >= 2 {
                            let _ = f.seek(io::SeekFrom::End(-2));
                            let mut buf = [0u8; 2];
                            if f.read_exact(&mut buf).is_ok() && buf == [b'\n', b'\n'] {
                                file_len - 1
                            } else {
                                file_len
                            }
                        } else {
                            file_len
                        }
                    }
                    Err(_) => 0,
                };

                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(false)
                    .open(path)?;

                file.seek(io::SeekFrom::Start(write_start))?;

                let start_idx = shell.history_append_index;
                for line in shell.histories.iter().skip(start_idx) {
                    writeln!(file, "{}", line)?;
                }

                shell.history_append_index = shell.histories.len();

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

fn execute_exit(args: &[String]) -> Result<i32, ShellError> {
    let exit_code = args
        .first()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);

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
