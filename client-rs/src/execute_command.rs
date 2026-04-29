use shlex::split;
use std::process::{Command, Output};
/// # Command Execution Module
///
/// This module provides functionality for executing shell commands from the backup client.
/// It serves as a secure bridge between the backup system and the host operating system,
/// allowing controlled execution of commands that might be needed for backup or maintenance tasks.
///
/// The module implements proper command parsing, argument handling, and result capturing
/// to ensure secure and reliable command execution.
use tracing::{debug, info, warn};

/// Executes a shell command and returns the output.
///
/// This function parses the provided command string using shell-like syntax rules,
/// executes it as a separate process, and captures its output (stdout, stderr) and exit status.
///
/// # Arguments
///
/// * `command` - A string slice that represents the command to be executed.
///   This can include arguments and supports shell-like syntax for argument splitting.
///
/// # Returns
///
/// Returns a `std::io::Result<Output>` containing:
/// - The exit status of the command
/// - The stdout output as a byte vector
/// - The stderr output as a byte vector
///
/// # Errors
///
/// Returns an error if:
/// - The command cannot be parsed correctly
/// - The command is empty after parsing
/// - The executable cannot be found
/// - The process fails to start
/// - There are permission issues
/// - Other I/O errors occur during execution
///
/// # Security Considerations
///
/// This function executes commands with the same privileges as the calling process.
/// Care should be taken to validate and sanitize command input to prevent security issues,
/// especially when the command source is external or user-provided.
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
