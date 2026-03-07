/// Handles removing backups.
pub mod remove;
/// Handles removing backup references.
pub mod remove_machine;
/// Manages the state of backup removal operations.
pub mod remove_state;
/// Handles restoring backups.
pub mod restore;
/// Handles restoring backup references.
pub mod restore_machine;
/// Manages the state of backup restoration operations.
pub mod restore_state;
/// Retention policy: classifies backups into calendar slots and identifies deletions.
pub mod retention;
/// Handles saving backups.
pub mod save;
/// Handles saving backup references.
pub mod save_machine;
/// Manages the state of backup saving operations.
pub mod save_state;
