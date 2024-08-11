To perform backups, you need to install an agent on your computer. Please download the agent
corresponding to your operating system.

After downloading, you will have a zip file containing the following:

* `config.yml`: The configuration file for the agent.
* `ws_client_daemon`: The daemon executable for Linux.
* Certificates necessary for authentication.

Extract the contents of the zip file to a folder of your choice. For example, you can extract it to
the folder `/opt/woodstock`. Then, follow these steps to install the agent as a Linux service:

1. Open a terminal with administrative privileges.
2. Navigate to the folder where you extracted the files. For example:

    ```bash
    cd /opt/woodstock
    ```

3. Make the daemon executable:

    ```bash
    chmod +x ws_client_daemon
    ```

4. Create a systemd service file. Open a text editor with administrative privileges and create a
   file at `/etc/systemd/system/woodstock.service` with the following content:

    ```systemd
    [Unit]
    Description=Woodstock Backup Client
    After=network.target

    [Service]
    ExecStart=/opt/woodstock/ws_client_daemon --config /opt/woodstock
    Restart=always
    User=nobody
    Group=nogroup

    [Install]
    WantedBy=multi-user.target
    ```

5. Reload the systemd daemon to recognize the new service and start it:

    ```bash
    sudo systemctl daemon-reload
    sudo systemctl enable woodstock.service
    sudo systemctl start woodstock.service
    ```

6. Enable the service to start on boot with the following command:

The agent is now installed and running as a Linux service. It will automatically start with your
computer and listen for instructions from the server.
