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

## Contribution

- I accept contribution :)
- The rules is simple: i can accept it, or refuse it depending on
  - Is the feature is in the goal of this backup application ? (i don't accept contribution to make coffee when backup is made)
  - Is the feature is write as i think it should be write (very very subjective, but i will comment your pull request to tell you how i think it should be written)

If you want to add a new feature, the better way is to ask me if i'm not already made it and if i accept pull request on it before starting.

PS: I know that this make me a Benevolent Dictor for Life, but if i have many perfect contribution, maybe that can be changed :)

## Licence

[MIT](https://choosealicense.com/licenses/mit/)
