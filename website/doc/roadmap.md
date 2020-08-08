# Roadmap

This is the feature i would like to develop :

- More unit test and e2e testing
- Maybe replace btrfs because
  - Not working without it
  - Performances issues when there is too many snapshot (not experimented but i read it).
  - Some statitistics that btrfs can't give me easily (as really shared size, and not shared size by host)
  - Not possible to make deduplication over multiple host
  - Make it possible to run the application on multiple server ?
- If i replace btrfs, how to store backup and make deduplication with rsync ?? Maybe i need to write my own backup client ?
- Maybe use Vue 3 composition API
- Automatic deletion of backup (based on scheduler)
