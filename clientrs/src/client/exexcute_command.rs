use log::{debug, info, warn};
use shlex::split;
use std::process::{Command, Output};

/// Executes a command and returns the output.
///
/// # Arguments
///
/// * `command` - A string slice that represents the command to be executed.
///
/// # Returns
///
/// Returns a `std::io::Result` containing the output of the executed command.
///
/// # Errors
///
/// The command generate an error if the command can't be executed.
///
pub fn execute_command(command: &str) -> Result<Output, std::io::Error> {
    debug!("Execute command: {}", command);

    let command = split(command).unwrap_or_else(|| vec![command.to_string()]);
    if command.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid command",
        ));
    }

    let mut command_to_execute = Command::new(&command[0]);
    for arg in &command[1..] {
        command_to_execute.arg(arg);
    }
    let output = command_to_execute.output()?;

    if output.status.success() {
        info!("Command executed successfully: {}", command.join(" "));
    } else {
        warn!(
            "Command failed with exit code {}: {}",
            output.status.code().unwrap_or(-1),
            command.join(" ")
        );
    }

    Ok(output)
}
