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
    echo "deb [signed-by=/etc/apt/keyrings/gitea-ShadowareOrg.asc] https://gogs.shadoware.org/api/packages/ShadowareOrg/debian $distribution $component" | sudo tee -a /etc/apt/sources.list.d/gitea-shadowareorg.list
    sudo apt update
    ```

3. Install the agent package:

    ```bash
    sudo apt install woodstock-client-rs
    ```

The agent is now installed and running as a Linux service. It will automatically start with your
computer and listen for instructions from the server.
