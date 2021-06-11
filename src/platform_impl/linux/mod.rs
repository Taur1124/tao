// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0

#![cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]

mod clipboard;
mod event_loop;
mod menu;
mod monitor;
#[cfg(feature = "tray")]
mod system_tray;
mod window;

#[cfg(feature = "tray")]
pub use self::system_tray::{SystemTray, SystemTrayBuilder};
pub use self::{
  clipboard::Clipboard,
  menu::{Menu, MenuItemAttributes},
};
pub use event_loop::{EventLoop, EventLoopProxy, EventLoopWindowTarget};
pub use monitor::{MonitorHandle, VideoMode};
pub use window::{
  hit_test, PlatformIcon, PlatformSpecificWindowBuilderAttributes, Window, WindowId,
};

#[derive(Debug, Clone)]
pub struct OsError;

impl std::fmt::Display for OsError {
  fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    Ok(())
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceId(usize);

impl DeviceId {
  pub unsafe fn dummy() -> Self {
    Self(0)
  }
}
