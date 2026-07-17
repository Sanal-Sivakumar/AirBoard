#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
pub mod desktop;

#[cfg(target_os = "android")]
pub mod android;
