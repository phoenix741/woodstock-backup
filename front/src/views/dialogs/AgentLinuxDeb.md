To perform backups, you need to install an agent on your computer. Please download the agent
corresponding to your operating system.

After downloading, you will have a zip file containing the following:

* `config.yml`: The configuration file for the agent.
* Certificates necessary for authentication.

Extract the contents of the zip file to `/etc/woodstock/`. Then, follow these steps to install the agent as a Linux service:

1. Open a terminal with administrative privileges.
2. Install the source package with the following commands:

    ```bash
    sudo curl https://gogs.shadoware.org/api/packages/ShadowareOrg/debian/repository.key -o /etc/apt/keyrings/gitea-ShadowareOrg.asc
    echo "deb [signed-by=/etc/apt/keyrings/gitea-ShadowareOrg.asc] https://gogs.shadoware.org/api/packages/ShadowareOrg/debian bookworm main" | sudo tee -a /etc/apt/sources.list.d/gitea-shadowareorg.list
    sudo apt update
    ```

3. Install the agent package:

    ```bash
    sudo apt install woodstock-client-rs
    ```

The agent is now installed and running as a Linux service. It will automatically start with your
computer and listen for instructions from the server.

## Configuration Options

The main configuration is done in the `config.yml` file located in `/etc/woodstock/`. Here are all the available options:

* `hostname`: The name that identifies this host on the server (defaults to system hostname)
* `bind`: The network address to bind to (default: "0.0.0.0:3657")
* `password`: Password for authentication with the server
* `secret`: The secret key for the client (defaults to a randomly generated 64-byte hexadecimal string)
* `backup_timeout`: Timeout for backup operations in seconds (default: 3600 seconds - 1 hour)
* `max_backup_seconds`: Maximum duration of a backup operation in seconds (default: 43200 seconds - 12 hours)
* `resolution_mode`: Connection method to the server - "Direct" (default), "Mdns" (if compiled with mDNS support), or "None"
* `mdns_interfaces`: Optional list of network interfaces to use for mDNS discovery
* `server`: Server address when using Direct mode (required if resolution_mode is Direct)
* `disable_restauration`: Set to true to disable restore capabilities for security reasons (default: false)
* `xattr`: Set to true to enable backup of extended attributes (default: false)
* `acl`: Set to true to enable backup of ACLs (default: false)
* `auto_update`: Enable automatic updates (default: false on Linux)
* `update_delay`: Time in seconds between update checks (default: 86400 - 24 hours)
* `log_directory`: Directory where logs will be stored

You can also start the service with the `--config-dir` parameter to specify an alternative configuration directory, or set the `CLIENT_PATH` environment variable.
