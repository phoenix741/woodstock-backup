To perform backups, you need to install an agent on your computer. Please download the agent
corresponding to your operating system.

After downloading, you will have a zip file containing the following:

* `config.yml`: The configuration file for the agent.
* `ws_client_daemon.exe`: The daemon executable for Windows.
* Certificates necessary for authentication.

Extract the contents of the zip file to a folder of your choice. For example, you can extract it to
the folder `C:\ProgramData\woodstock`. Then, follow these steps to install the agent as a Windows
service:

1. Open a Command Prompt with administrative privileges.
2. Navigate to the folder where you extracted the files. For example:

    ```powershell
    cd C:\ProgramData\woodstock
    ```

3. Install the agent as a service using the following command:

    ```powershell
    .\ws_client_daemon.exe --config-dir C:\ProgramData\woodstock install-service
    ```

The agent is now installed and running as a Windows service. It will automatically start with your
computer and listen for instructions from the server.
