//! Ctrl+C wiring shared by every long-running `cli-rs` command.

use tokio_util::sync::CancellationToken;

/// Returns a [`CancellationToken`] that cancels itself as soon as the user
/// presses Ctrl+C, so a long-running command (restore, sync, archive
/// run-now, fsck, mDNS resolve) can stop cleanly through its normal cancel
/// path instead of the process being killed outright with no cleanup.
pub fn cancellation_token_with_ctrl_c() -> CancellationToken {
    let token = CancellationToken::new();
    let watcher_token = token.clone();
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        watcher_token.cancel();

        println!("\nCtrl+C pressed, cancelling...");

        // Registering this handler replaces SIGINT's default (fatal)
        // disposition for the rest of the process. Phases that don't poll
        // the token (final output, teardown after the cancelled operation
        // returns) would otherwise become unkillable by Ctrl+C. A second
        // press forces the exit those phases used to get for free.
        let _ = tokio::signal::ctrl_c().await;
        std::process::exit(130);
    });
    token
}
