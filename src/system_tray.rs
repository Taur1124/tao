// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0

//! **UNSTABLE** -- The `SystemTray` struct and associated types.
//!
//! Use [SystemTrayBuilder][tray_builder] to create your tray instance.
//!
//! [ContextMenu][context_menu] is used to created a Window menu on Windows and Linux. On macOS it's used in the menubar.
//!
//! ```rust,ignore
//! let mut tray_menu = ContextMenu::new();
//! let icon = include_bytes!("my_icon.png").to_vec();
//!
//! tray_menu.add_item(MenuItemAttributes::new("My menu item"));
//!
//! let mut system_tray = SystemTrayBuilder::new(icon, Some(tray_menu))
//!   .build(&event_loop)
//!   .unwrap();
//! ```
//!
//! # Linux
//! A menu is required or the tray return an error containing `assertion 'G_IS_DBUS_CONNECTION (connection)'`.
//!
//! [tray_builder]: crate::system_tray::SystemTrayBuilder
//! [menu_bar]: crate::menu::MenuBar
//! [context_menu]: crate::menu::ContextMenu

use crate::{
  error::OsError,
  event_loop::EventLoopWindowTarget,
  menu::Menu,
  platform_impl::{
    SystemTray as SystemTrayPlatform, SystemTrayBuilder as SystemTrayBuilderPlatform,
  },
};

pub use crate::icon::{BadIcon, Icon};

/// Object that allows you to build SystemTray instance.
pub struct SystemTrayBuilder(pub(crate) SystemTrayBuilderPlatform);

#[cfg(target_os = "linux")]
use std::path::PathBuf;

impl SystemTrayBuilder {
  /// Creates a new SystemTray for platforms where this is appropriate.
  pub fn new(icon: Icon, menu: Option<Menu>) -> Self {
    Self(SystemTrayBuilderPlatform::new(icon, menu))
  }

  /// Builds the SystemTray.
  ///
  /// Possible causes of error include denied permission, incompatible system, and lack of memory.
  pub fn build<T: 'static>(
    self,
    window_target: &EventLoopWindowTarget<T>,
  ) -> Result<SystemTray, OsError> {
    self.0.build(window_target)
  }
}

/// Represents a System Tray instance.
pub struct SystemTray(pub SystemTrayPlatform);

impl SystemTray {
  /// Set new tray icon.
  pub fn set_icon(&mut self, icon: Icon) {
    self.0.set_icon(icon)
  }

  /// Set new tray menu.
  pub fn set_menu(&mut self, menu: Option<Menu>) {
    self.0.set_menu(menu)
  }
}
