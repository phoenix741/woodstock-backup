# F.A.Q.

## Why not using ... ?

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

## Why the name Woodstock backup ?

Because, find a name for an application is the thing the most complicated on development. When i start to wrote this application,
i watching the first episode of the season 4 of _Legends of Tomorrow_ and i found the name fun :)

## Why to wrote it in Node.JS ?

Short: Because

Long: I have hesitate between Go, NodeJS, .... But the core of the program is btrfs and rsync. The use of NodeJS have no impact on
the performance, or the stability. The front should be dynamic so to have Javascript. So why not the server in Javascript ?
