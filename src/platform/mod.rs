#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub use windows::capture_screen;

#[cfg(target_os = "macos")]
pub use macos::capture_screen;

#[cfg(target_os = "linux")]
pub use linux::capture_screen;

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub mod fallback;

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub use fallback::capture_screen;