# Documentation

_Woodstock backup_ is a backup software. The purpose is to backup devices to a central host.

Instead of launching the backup from the client, it's the server that contacts the client to launch the backup.

When possible, the client can create a filesystem snapshot before scanning files. This gives the backup
worker a stable view of the data while the machine continues to run.

This documentation will explain how to install, configure and use _Woodstock backup_. The Internal part of
the documentation will explain how the software works internally for development purposes.

## Summary

- [Installation](/doc/installation)
- [Agent](/doc/agent)
- [Configuration](/doc/configuration)
- [Update the scheduler](/doc/scheduler)
- [FAQ](/doc/faq)
- [Roadmap](/doc/roadmap)
- [Migration](/doc/migration/)
  - [From BackupPC](/doc/migration/backuppc)
- [Internal](/doc/internal/)
  - [Pool](/doc/internal/pool)
  - [Snapshot-backed backups](/doc/internal/snapshots)
  - [Server Authentication to Client](/doc/internal/client_auth_backup)
  - [Client Authentication to Server](/doc/internal/client_auth_dns)
  - [Colors](/doc/colors)

## Contribution

If you want to contribute to the project, you can open an issue or a pull request. You can also contact me
to discuss the project and check if I haven't already started development of the feature.

## License

[MIT](https://choosealicense.com/licenses/mit/)
