# Roadmap

These are the features I would like to develop :

- More unit tests and e2e testing
- Maybe replace btrfs because
  - Not working without it
  - Performances issues when there are too many snapshots (not experimented but I read it).
  - Some statistics that btrfs can't give me easily (as really shared size, and not shared size by host)
  - Not possible to make deduplication over multiple hosts
  - Make it possible to run the application on multiple servers ?
- If I replace btrfs, how to store backup and make deduplication with rsync ?? Maybe I need to write my own backup client ?
- Maybe use Vue 3 compositions API
- Automatic deletion of backup (based on scheduler)
