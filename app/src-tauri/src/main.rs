// Hides the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "linux")]
    configure_linux_graphics();
    tinta_app::run();
}

/// `WebKitGTK` crashes with "Error 71 (Protocol error)" on NVIDIA + Wayland
/// when explicit sync is enabled. Disabling it keeps GPU acceleration on.
/// The variable is only read by the NVIDIA driver, so it is harmless on
/// other GPUs. Set before any thread starts, as `set_var` requires.
#[cfg(target_os = "linux")]
fn configure_linux_graphics() {
    const NVIDIA_EXPLICIT_SYNC: &str = "__NV_DISABLE_EXPLICIT_SYNC";
    if std::env::var_os(NVIDIA_EXPLICIT_SYNC).is_none() {
        std::env::set_var(NVIDIA_EXPLICIT_SYNC, "1");
    }
}
