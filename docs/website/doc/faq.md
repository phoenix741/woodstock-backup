# F.A.Q

## Why not use ... ?

There are many software solutions for computer backup:

- **BackupPC**: My favorite.

  But I wanted to make a backup of each device's backup on a USB drive (to store in another location) and I wanted to access it
  without untarring an archive. To do this, I used a personal script that mounts the USB drive, mounts the BackupPC pool with
  backuppcfs-v4.pl and performs rsync.

  However, when using backuppcfs-v4.pl, I encountered problems with permissions on Windows client backups (I had to modify the script
  to deactivate permission handling), and issues when copying large files (e.g., 260GB)

- **UrBackup**: Another source of inspiration. UrBackup is able to use Btrfs to manage snapshots.

- **Borg**: I love the concept, but I wanted the server to be able to decrypt the backup to archive it on a USB drive (along with all
  other backups).

  I also wanted a nice UI to list host backups in a centralized way.

- There are many backup applications that work when launched from the client computer to a USB drive or over the network,
  but it's the responsibility of the client to set them up.

So I decided to write my own backup program. Because why not.

## Why the name `Woodstock backup`?

Because finding a name for an application is the most complicated part of the development process. When I started to
write this application, I was watching the first episode of season 4 of _Legends of Tomorrow_ and I found the name
fun :)

The backups are stored as chunks in the pool directory. This also makes me think of a stock of wood.

## Why Rust?

The server and agent are written entirely in **Rust**.

The project started as a proof-of-concept in **C++**, but multi-threading complexity made rapid iteration difficult. It was then rewritten in **Node.js** for a quicker prototype. After hitting performance bottlenecks, the critical components were migrated to Rust one by one — first the agent, then the storage core, and finally the whole server.

Today the entire backend is in Rust, which gives:

- Predictable memory usage and no garbage-collector pauses during large backups
- Native performance for chunk hashing (Blake3 with parallelism) and compression (Zstd)
- A single statically-linked binary per component, easy to deploy in containers or on bare metal
