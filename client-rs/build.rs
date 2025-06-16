//! Build script for the Woodstock Backup client
//!
//! This script handles platform-specific build configurations:
//! - On Windows: Sets application icon and embeds version information

// Import the windows_exe_info crate only on Windows targets
#[cfg(target_os = "windows")]
extern crate windows_exe_info;

fn main() {
    // Windows-specific configuration
    #[cfg(target_os = "windows")]
    {
        // Set the application icon from the .ico file
        windows_exe_info::icon::icon_ico("../winres/woodstock.ico");

        // Link version information from Cargo environment variables
        // This embeds details like version number, product name, etc. into the executable
        windows_exe_info::versioninfo::link_cargo_env();
    }
}
