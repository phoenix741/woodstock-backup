# F.A.Q

## Why not using ... ?

There are many softwares that do computer backup:

- **BackupPC**: My favorite.

  But I want to make a backup of each device backup on usbdrive (to put in another location) and I want to access it
  without untar an archive. To do this, I use a personal script that mount the usb drive, mount backuppc pool with
  backuppcfs-v4.pl and make a rsync.

  But when I use backuppcfs-v4.pl, I have some problem with permissions and backup of windows client (I change the script
  to deactivate permission), and for copying big file: 260Gb

- **UrBackup**: Another source of inspiration. UrBackup is able to use btrfs to manage snapshot.

- **Borg**: I love the concept, but I want that the server can decrypt the backup to archive it on usb drive (with all
  other backup).

  I want a cool IHM to list host backup too, in a centralized way.

- There is many backup application that work when launched from the client computer on a usbdrive or on the network,
  but that is the responsability of the client to setup it.

So I decided to write my own backup program. Because why not.

## Why the name `Woodstock backup` ?

Because finding a name for an application is the most complicated thing in the development process. When I started to
write this application, I was watching the first episode of season 4 of _Legends of Tomorrow_ and I found the name
fun :).

The backup are stored as chunks in the pool directory. This make me think of a stock of wood too.

## Why using Node.JS ?

Short: Because

Long: I hesitated between Go, NodeJS... I have start writting the program in `C++` for performance, but the thread
management make it writting a proof of concept complicated. So i start to write it in `NodeJS` because it is easy to
quickly write a proof of concept.

After some test, i experiment performance problem with `NodeJS`. So i rewrite the core of the program in `Rust`. I write
the client in `Rust` too. The only part of the program that is in JavaScript is the front, the controller, and the
resolver.

Maybe a day i rewrite the program in full `Rust`.
