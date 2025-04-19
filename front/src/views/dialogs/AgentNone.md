To perform backups, you need to install an agent on your computer. Please download the agent
corresponding to your operating system.

After downloading, you will have a zip file containing the following:

- `config.yml`: The configuration file for the agent.
- Certificates necessary for authentication.

Extract the contents of the zip file to a folder of your choice. Download the agent binaries from the release page: <https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/releases> and install it according to your operating system's requirements.

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
- `xattr`: Set to true to enable backup of extended attributes on Linux (default: false)
- `acl`: Set to true to enable backup of ACLs on Linux (default: false)
- `auto_update`: Enable automatic updates (default: true on Windows, false on other platforms)
- `update_delay`: Time in seconds between update checks (default: 86400 - 24 hours)
- `log_directory`: Directory where logs will be stored

You can also start the agent with the `--config-dir` parameter to specify an alternative configuration directory, or set the `CLIENT_PATH` environment variable.
