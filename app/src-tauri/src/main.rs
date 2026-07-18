#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  // WebKitGTK 4.1 DMA-BUF renderer requires EGL; on Arch/NixOS it
  // aborts with EGL_BAD_PARAMETER.  Fallback to a compatible path without
  // forcing software rendering.  Respect an explicit user override.
  #[cfg(target_os = "linux")]
  if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
  }

  pgpilot_lib::run();
}
