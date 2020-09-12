# Installation

## What are the prerequisites?

**Woodstock backup** needs the following softwares to work :

- Redis: To manage the queue of backup
- Btrfs: To manage the storage of backup (ideally the mount point should be dedicated to the backup storage)
- NodeJS 10: To run the application
- The transpiled application: without it, it can't work

Theorically if the btrfs storage is shared, it's possible to run multiple instances of the same application on the same server.

## Install with docker

-`Docker` needs redis to store bull queue.

- The backup storage should be a btrfs volume.
- The docker image need `SYS_ADMIN` capability.

```yaml
version: "2"

services:
  woodstock:
    image: phoenix741/woodstock-backup:1
    ports:
      - 3000:3000
    links:
      - redis
    environment:
      - REDIS_HOST=redis
      - REDIS_PORT=6379
      - MAX_BACKUP_TASK=2
      - NODE_ENV=production
    volumes:
      - "backups_storage:/backups"
    cap_add:
      - SYS_ADMIN
  redis:
    image: "bitnami/redis:5.0"
    environment:
      # ALLOW_EMPTY_PASSWORD is recommended only for development.
      - ALLOW_EMPTY_PASSWORD=yes
      - REDIS_DISABLE_COMMANDS=FLUSHDB,FLUSHALL
    ports:
      - "6379:6379"
    volumes:
      - "redis_data:/bitnami/redis/data"

volumes:
  redis_data:
  backups_storage:
    driver: local
    driver_opts:
      type: none
      device: /var/lib/woodstock/
      o: bind
```

## Install with a deb package

To install woodstock-backup on a debian system, download the deb package.

- The `/var/lib/woodstock` directory needs to be a btrfs volume.
- Woodstock has a dependencie on redis to store bull queue.

```bash
sudo dpkg -i woodstock-backup_1.0.0_all.deb
sudo nano /etc/woodstock-backup/default
```

You can edit environment variable necessary to launch woodstock.

## Install manually

First install the linux distribution of your choice that satisfy the previous criteria. On this distribution install redis, and nodejs.
_It's important to secure the connection of redis to avoid everybody to connect to your redis instance._

```bash
apt install debian:10
# The previous line is a joke ;)
# but in the next, example will be based on debian.
apt install redis nodejs
```

Ensure you have a mount point that use btrfs, ideally the mount point should be dedicated to the backup.

If you want the woodstock backup not be executed as root, you must add attribute **user_subvol_rm_allowed**.
You can activate the compression of btrfs depending or not.

```bash
mkfs.btrfs /dev/sdXYY
echo << EOF>> /etc/fstab
/dev/sdXYY /var/lib/woodstock btrfs rw,noatime,compress=zstd:9,user_subvol_rm_allowed,noauto  0  0
EOF
```

Clone and build the project

```bash
git clone https://gogs.shadoware.org/ShadowareOrg/woodstock-backup.git woodstock-backup

cd client
npm i
npm run build -- --prod

cd ../server
npm i
npm run build

cp ./config/woodstock-backup.service /etc/systemd/user/woodstock-backup.service
# HERE: You should edit the file /etc/systemd/user/woodstock-backup.systemd to put
# the environment variable with the value of your configuration
sudo systemctl daemon-reload
sudo systemctl enable woodstock-backup
sudo systemctl start woodstock-backup
```

The available environment variables are :

- **STATIC_PATH**: should be the path of the file in the client directory (example: `/opt/woodstock/client/dist`)
- **BACKUP_PATH**: should be the path of the storage (on a btrfs drive, example ̀`var/lib/woodstock/woodstock`
- **REDIS_HOST**: the host where redis is installed
- **REDIS_PORT**: the port of redis to connect
