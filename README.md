# Woodstock Backup

_Woodstock backup_ is a backup software. The purpose is to backup software from a central point.

Instead of launching the backup from the client, it's the server that contact the client to launch the backup.

The actual version is based on

- rsync to copy file from the client to the server
- btrfs to create snapshot and shared block not actually modified

## Why not using...

There are many software to backup computer:

- **BackupPC**: My favorite.

  But i want to create backup of the backup on usbdrive (to put in another location) and i want to access it without untar an archive. To do this, i use a personal script that mount the usb drive, mount backuppc pool with backuppcfs-v4.pl and make a rsync.

  But when i use backuppcfs-v4.pl, i have some problem with permissions and backup of windows client (i change the script to deactivate permission), and for copying big file : 260Gb

- **UrBackup**: Another source of inspiration. UrBackup is able to use btrfs to manage snapshot.

- **Borg**: I love the principe, but i want that the server can decrypt the backup to archive it on usb drive (with all other
  backup).

  I want a cool IHM to list host backup too, on a centralized way.

- There is many backup application that work when launched from the client computer on a usbdrive or on the network, but that is
  the responsability of the client to setup it.

So i decide to wrote my backup program. Because why not.

## How to install it

**What are pre-request ?**

**Woodstock backup** need the following software to work :

- Redis: To manage the queue of backup
- Btrfs: To manage the storage of backup (ideally the mount point should be dedicated to the backup storage)
- NodeJS 10: To run the application
- The transpiled application: without it, it can't work

Theorically if the btrfs storage is shared, it's possible to run multiple instance of the same application on the same server.

**Install with docker.**

- `Docker` need redis to store bull queue.
- The backup storage should be a btrfs volume.
- The docker image need `SYS_ADMIN` capability.

```yaml
version: "2"

services:
  woodstock:
    image: phoenix741/woodstock-backup:develop
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
    driver: local
  backups_storage:
    driver: local
    driver_opts:
      type: none
      device: /var/lib/woodstock/
      o: bind
```

**Install with a deb package.**

**Install manually.**

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
You can active the compression of btrfs depending or not.

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

The available environment variable are :

- **STATIC_PATH**: should be the path of the file in the client directory (example: `/opt/woodstock/client/dist`)
- **BACKUP_PATH**: should be the path of the storage (on a btrfs drive, example ̀`var/lib/woodstock/woodstock`
- **REDIS_HOST**: the host where redis is installed
- **REDIS_PORT**: the port of redis to connect

## FAQ

**Why the name Woodstock backup ?**

Because, find a name for an application is the thing the most complicated on development. When i start to wrote this application,
i watching the first episode of the season 4 of _Legends of Tomorrow_ and i found the name fun :)

**Why to wrote it in Node.JS ?**

Short: Because

Long: I have hesitate between Go, NodeJS, .... But the core of the program is btrfs and rsync. The use of NodeJS have no impact on
the performance, or the stability. The front should be dynamic so to have Javascript. So why not the server in Javascript ?

## Roadmap

This is the feature i would like to develop :

- More unit test and e2e testing
- Maybe replace btrfs because
  - Not working without it
  - Performances issues when there is too many snapshot (not experimented but i read it).
  - Some statitistics that btrfs can't give me easily (as really shared size, and not shared size by host)
  - Not possible to make deduplication over multiple host
  - Make it possible to run the application on multiple server ?
- If i replace btrfs, how to store backup and make deduplication with rsync ?? Maybe i need to write my own backup client ?
- Maybe use Vue 3 composition API

## Contribution

- I accept contribution :)
- The rules is simple: i can accept it, or refuse it depending on
  - Is the feature is in the goal of this backup application ? (i don't accept contribution to make coffee when backup is made)
  - Is the feature is write as i think it should be write (very very subjective, but i will comment your pull request to tell you how i think it should be written)

If you want to add a new feature, the better way is to ask me if i'm not already made it and if i accept pull request on it before starting.

PS: I know that this make me a Benevolent Dictor for Life, but if i have many perfect contribution, maybe that can be changed :)

## Licence

[MIT](https://choosealicense.com/licenses/mit/)
