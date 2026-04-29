To perform backups, you need to install an agent on your computer. Please download the agent
corresponding to your operating system.

After downloading, you will have a zip file containing the following:

- `config.yml`: The configuration file for the agent.
- `ws_client_daemon.exe`: The daemon executable for Windows.
- Certificates necessary for authentication.

Extract the contents of the zip file to a folder of your choice. For example, you can extract it to
the folder `C:\ProgramData\woodstock`. Then, follow these steps to install the agent as a Windows
service:

1. Open a Command Prompt with administrative privileges.
2. Navigate to the folder where you extracted the files. For example:

   ```powershell
   cd C:\ProgramData\woodstock
   ```

3. Configure the Windows Firewall to allow Woodstock agent communication:

   ```powershell
   .\ws_client_daemon.exe --config-dir C:\ProgramData\woodstock install-fw-rule
   ```

4. Install the agent as a service using the following command:

   ```powershell
   .\ws_client_daemon.exe --config-dir C:\ProgramData\woodstock install-service
   ```

The agent is now installed and running as a Windows service. It will automatically start with your
computer and listen for instructions from the server.

## Configuration Options

The main configuration is done in the `config.yml` file. Here are all the available options:

- `hostname`: The name that identifies this host on the server (defaults to system hostname)
- `bind`: The network address to bind to (default: "0.0.0.0:3657")
- `password`: Password for authentication with the server
- `secret`: The secret key for the client (defaults to a randomly generated 64-byte hexadecimal string)
- `backup_timeout`: Timeout for backup operations in seconds (default: 3600 seconds - 1 hour)
- `max_backup_seconds`: Maximum duration of a backup operation in seconds (default: 43200 seconds - 12 hours)
- `resolution_mode`: Connection method to the server - "Direct" (default), "Mdns" (if compiled with mDNS support), or "None"
- `mdns_interfaces`: Optional list of network interfaces to use for mDNS discovery
- `server`: Server address when using Direct mode (required if resolution_mode is Direct)
- `disable_restauration`: Set to true to disable restore capabilities for security reasons (default: false)
- `auto_update`: Enable automatic updates (default: true on Windows)
- `update_delay`: Time in seconds between update checks (default: 86400 - 24 hours)
- `log_directory`: Directory where logs will be stored

You can also set the CLIENT_PATH environment variable to specify an alternative configuration directory.
