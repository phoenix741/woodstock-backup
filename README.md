# Woodstock Backup

[![Build Status](https://ci.shadoware.org/api/badges/ShadowareOrg/woodstock-backup/status.svg)](https://ci.shadoware.org/ShadowareOrg/woodstock-backup)

_Woodstock backup_ is a backup software. The purpose is to backup software from a central point.

Instead of launching the backup from the client, it's the server that contacts the client to launch the backup.

The actual version is based on

- rsync to copy file from the client to the server
- btrfs to create snapshot and shared block not actually modified

## Documentation

You can find the [documentation online](https://woodstockbackup.shadoware.org).

## Licence

[MIT](https://choosealicense.com/licenses/mit/)
