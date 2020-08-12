# F.A.Q.

## Why not using ... ?

There are many softwares that do computer backup:

- **BackupPC**: My favorite.

  But I want to create backup of the backup on usbdrive (to put in another location) and I want to access it without untar an archive. To do this, I use a personal script that mount the usb drive, mount backuppc pool with backuppcfs-v4.pl and make a rsync.

  But when I use backuppcfs-v4.pl, I have some problem with permissions and backup of windows client (I change the script to deactivate permission), and for copying big file : 260Gb

- **UrBackup**: Another source of inspiration. UrBackup is able to use btrfs to manage snapshot.

- **Borg**: I love the concept, but I want that the server can decrypt the backup to archive it on usb drive (with all other
  backup).

  I want a cool IHM to list host backup too, in a centralized way.

- There is many backup application that work when launched from the client computer on a usbdrive or on the network, but that is
  the responsability of the client to setup it.

So I decided to write my own backup program. Because why not.

## Why the name Woodstock backup ?

Because finding a name for an application is the most complicated thing in the development process. When I started to write this application,
I was watching the first episode of season 4 of _Legends of Tomorrow_ and I found the name fun :)

## Why using Node.JS ?

Short: Because

Long: I hesitated between Go, NodeJS... But the core of the program is btrfs and rsync. The use of NodeJS has no impact on
the performance, or the stability. The front should be dynamic so to have Javascript. So why not the server in Javascript ?
