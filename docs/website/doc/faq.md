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

## Why use Node.JS?

Short: Because.

Long: I hesitated between Go, NodeJS, and others. I initially started writing the program in `C++` for performance, but thread
management made creating a proof of concept complicated. So I started to write it in `NodeJS` because it is easier to
quickly write a proof of concept.

After some testing, I encountered performance problems with `NodeJS`. So I rewrote the core of the program in `Rust`. I wrote
the client in `Rust` too. The only parts of the program that remain in JavaScript are the front-end, the controller, and the
resolver.

Maybe someday I'll rewrite the program entirely in `Rust`.
