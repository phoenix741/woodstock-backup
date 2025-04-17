#[cfg(target_os = "windows")]
extern crate windows_exe_info;

fn main() {
    #[cfg(target_os = "windows")]
    {
        use std::path::Path;

        windows_exe_info::icon::icon_ico(Path::new("../winres/woodstock.ico"));
        windows_exe_info::versioninfo::link_cargo_env();
    }
}
