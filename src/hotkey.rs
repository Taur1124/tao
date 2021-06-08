//! The HotKey struct and associated types.

use crate::{
  error::OsError,
  event_loop::EventLoopWindowTarget,
  keyboard::{Key, ModifiersState},
  platform_impl::{register_global_accelerators, GlobalAccelerator as GlobalAcceleratorPlatform},
};
use std::{
  borrow::Borrow,
  collections::hash_map::DefaultHasher,
  error, fmt,
  hash::{Hash, Hasher},
};

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAccelerator(pub(crate) GlobalAcceleratorPlatform);

impl GlobalAccelerator {
  pub fn test(self) {
    println!("TEST!!! {:?}", self.0);
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HotKeyManager {
  registered_hotkeys: Vec<GlobalAccelerator>,
}

impl Default for HotKeyManager {
  fn default() -> Self {
    Self {
      registered_hotkeys: Vec::new(),
    }
  }
}

impl HotKeyManager {
  pub fn new() -> Self {
    Default::default()
  }
  pub fn is_registered(&self, hotkey: &HotKey) -> bool {
    let hotkey = GlobalAccelerator(GlobalAcceleratorPlatform::new(hotkey.clone()));
    self.registered_hotkeys.contains(&Box::new(hotkey))
  }
  pub fn register(&mut self, hotkey: HotKey) -> Result<GlobalAccelerator, HotKeyManagerError> {
    if self.is_registered(&hotkey) {
      return Err(HotKeyManagerError::HotKeyAlreadyRegistered(hotkey));
    }
    let hotkey = GlobalAccelerator(GlobalAcceleratorPlatform::new(hotkey));
    self.registered_hotkeys.append(&mut vec![hotkey.clone()]);
    Ok(hotkey)
  }
  pub fn run<T: 'static>(
    &mut self,
    _window_target: &EventLoopWindowTarget<T>,
  ) -> Result<(), OsError> {
    register_global_accelerators(_window_target, &mut self.registered_hotkeys);
    println!("registered_hotkeys {:?}", self.registered_hotkeys);
    Ok(())
  }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct HotKey {
  pub(crate) mods: RawMods,
  pub(crate) key: Key,
}

impl HotKey {
  pub fn new(mods: impl Into<Option<RawMods>>, key: impl Into<Key>) -> Self {
    HotKey {
      mods: mods.into().unwrap_or(RawMods::None),
      key: key.into(),
    }
  }

  pub fn id(self) -> u16 {
    hash_hotkey_to_u16(self)
  }

  /// Returns `true` if this [`Key`] and [`ModifiersState`] matches this `HotKey`.
  ///
  /// [`Key`]: Key
  /// [`ModifiersState`]: crate::keyboard::ModifiersState
  pub fn matches(&self, modifiers: impl Borrow<ModifiersState>, key: impl Borrow<Key>) -> bool {
    // Should be a const but const bit_or doesn't work here.
    let base_mods =
      ModifiersState::SHIFT | ModifiersState::CONTROL | ModifiersState::ALT | ModifiersState::SUPER;
    let modifiers = modifiers.borrow();
    let key = key.borrow();
    self.mods == *modifiers & base_mods && self.key == *key
  }
}

/// Represents the platform-agnostic keyboard modifiers, for command handling.
///
/// **This does one thing: it allows specifying hotkeys that use the Command key
/// on macOS, but use the Ctrl key on other platforms.**
#[derive(Debug, Clone, Copy)]
pub enum SysMods {
  None,
  Shift,
  /// Command on macOS, and Ctrl on windows/linux
  Cmd,
  /// Command + Alt on macOS, Ctrl + Alt on windows/linux
  AltCmd,
  /// Command + Shift on macOS, Ctrl + Shift on windows/linux
  CmdShift,
  /// Command + Alt + Shift on macOS, Ctrl + Alt + Shift on windows/linux
  AltCmdShift,
}

/// Represents the active modifier keys.
///
/// This is intended to be clearer than [`ModifiersState`], when describing hotkeys.
///
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum RawMods {
  None,
  Alt,
  Ctrl,
  Meta,
  Shift,
  AltCtrl,
  AltMeta,
  AltShift,
  CtrlShift,
  CtrlMeta,
  MetaShift,
  AltCtrlMeta,
  AltCtrlShift,
  AltMetaShift,
  CtrlMetaShift,
  AltCtrlMetaShift,
}

impl std::cmp::PartialEq<ModifiersState> for RawMods {
  fn eq(&self, other: &ModifiersState) -> bool {
    let mods: ModifiersState = (*self).into();
    mods == *other
  }
}

impl std::cmp::PartialEq<RawMods> for ModifiersState {
  fn eq(&self, other: &RawMods) -> bool {
    other == self
  }
}

impl std::cmp::PartialEq<ModifiersState> for SysMods {
  fn eq(&self, other: &ModifiersState) -> bool {
    let mods: RawMods = (*self).into();
    mods == *other
  }
}

impl std::cmp::PartialEq<SysMods> for ModifiersState {
  fn eq(&self, other: &SysMods) -> bool {
    let other: RawMods = (*other).into();
    &other == self
  }
}

impl From<RawMods> for ModifiersState {
  fn from(src: RawMods) -> ModifiersState {
    let (alt, ctrl, meta, shift) = match src {
      RawMods::None => (false, false, false, false),
      RawMods::Alt => (true, false, false, false),
      RawMods::Ctrl => (false, true, false, false),
      RawMods::Meta => (false, false, true, false),
      RawMods::Shift => (false, false, false, true),
      RawMods::AltCtrl => (true, true, false, false),
      RawMods::AltMeta => (true, false, true, false),
      RawMods::AltShift => (true, false, false, true),
      RawMods::CtrlMeta => (false, true, true, false),
      RawMods::CtrlShift => (false, true, false, true),
      RawMods::MetaShift => (false, false, true, true),
      RawMods::AltCtrlMeta => (true, true, true, false),
      RawMods::AltMetaShift => (true, false, true, true),
      RawMods::AltCtrlShift => (true, true, false, true),
      RawMods::CtrlMetaShift => (false, true, true, true),
      RawMods::AltCtrlMetaShift => (true, true, true, true),
    };
    let mut mods = ModifiersState::empty();
    mods.set(ModifiersState::ALT, alt);
    mods.set(ModifiersState::CONTROL, ctrl);
    mods.set(ModifiersState::SUPER, meta);
    mods.set(ModifiersState::SHIFT, shift);
    mods
  }
}

// we do this so that HotKey::new can accept `None` as an initial argument.
impl From<SysMods> for Option<RawMods> {
  fn from(src: SysMods) -> Option<RawMods> {
    Some(src.into())
  }
}

impl From<SysMods> for RawMods {
  fn from(src: SysMods) -> RawMods {
    #[cfg(target_os = "macos")]
    match src {
      SysMods::None => RawMods::None,
      SysMods::Shift => RawMods::Shift,
      SysMods::Cmd => RawMods::Meta,
      SysMods::AltCmd => RawMods::AltMeta,
      SysMods::CmdShift => RawMods::MetaShift,
      SysMods::AltCmdShift => RawMods::AltMetaShift,
    }
    #[cfg(not(target_os = "macos"))]
    match src {
      SysMods::None => RawMods::None,
      SysMods::Shift => RawMods::Shift,
      SysMods::Cmd => RawMods::Ctrl,
      SysMods::AltCmd => RawMods::AltCtrl,
      SysMods::CmdShift => RawMods::CtrlShift,
      SysMods::AltCmdShift => RawMods::AltCtrlShift,
    }
  }
}
#[derive(Debug)]
pub enum HotKeyManagerError {
  HotKeyAlreadyRegistered(HotKey),
  HotKeyNotRegistered(HotKey),
  InvalidHotKey(String),
}
impl error::Error for HotKeyManagerError {}
impl fmt::Display for HotKeyManagerError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    match self {
      HotKeyManagerError::HotKeyAlreadyRegistered(e) => {
        f.pad(&format!("hotkey already registered: {:?}", e))
      }
      HotKeyManagerError::HotKeyNotRegistered(e) => {
        f.pad(&format!("hotkey not registered: {:?}", e))
      }
      HotKeyManagerError::InvalidHotKey(e) => e.fmt(f),
    }
  }
}

fn hash_hotkey_to_u16(hotkey: HotKey) -> u16 {
  let mut s = DefaultHasher::new();
  hotkey.hash(&mut s);
  s.finish() as u16
}
