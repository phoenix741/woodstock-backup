# Roadmap

These are the features I would like to develop:

- Additional snapshot backends (LVM, ZFS) and explicit per-job snapshot policy controls
- More unit tests and end-to-end testing
- Custom chunk size configuration
- Options to handle hash collisions (currently Blake3 is used; collision probability is negligible but not zero)
