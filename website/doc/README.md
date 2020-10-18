# Documentation

_Woodstock backup_ is a backup software. The purpose is to backup software from a central point.

Instead of launching the backup from the client, it's the server that contact the client to launch the backup.

The actual version is based on

- rsync to copy file from the client to the server
- btrfs to create snapshot and shared block not actually modified

## Summary

- [Installation](/doc/installation/)
- [Add new host in configuration](/doc/addnewhost/)
- [Update tools](/doc/updatetools/)
- [Update the scheduler](/doc/updatescheduler/)
- [FAQ](/doc/faq/)
- [Roadmap](/doc/roadmap/)
- [Internal](/doc/internal/)

## Contribution

- I accept contributions :)
- The rules are simple: I can accept or refuse depending on
  - Is the feature in line with the goal of this backup application ? (I don't accept contributions to make coffee when backup is made)
  - Is the feature written the way I think it should be written (very very subjective, but I will comment your pull request to tell you how I think it should be written)

If you want to add a new feature, the best way is to ask me first if I haven't already made it and if I would accept pull request on it before starting.

PS: I know that this make me a Benevolent Dictor for Life, but if I receive a lot of great contributions, maybe that can be changed :)

## Licence

[MIT](https://choosealicense.com/licenses/mit/)
