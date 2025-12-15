use crate::{error::ShellError, parser::ast::Command, shell::Shell};

const BUILTINS: &[&str] = &["exit", "echo", "type"];

pub fn is_builtin(program: &str) -> bool {
    BUILTINS.contains(&program)
}

pub fn execute_builtin(_shell: &mut Shell, command: &Command) -> Result<i32, ShellError> {
    match command.program.as_str() {
        "exit" => execute_exit(&command.arguments),
        "echo" => execute_echo(&command.arguments),
        "type" => execute_type(&command.arguments),
        _ => Err(ShellError::CommandNotFound(format!(
            "{}: command not found",
            command.program
        ))),
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
    println!("{}", args.join(" "));
    Ok(0)
}

fn execute_type(args: &[String]) -> Result<i32, ShellError> {
    if args.is_empty() {
        return Err(ShellError::IntenalError(
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
        .map_err(|_| ShellError::IntenalError("Not valid UTF-8".to_string()))?;

    print!("{} is {}", program, output);
    Ok(0)
}
