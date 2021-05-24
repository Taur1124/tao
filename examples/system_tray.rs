// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0

#[cfg(any(
  target_os = "macos",
  target_os = "windows",
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
fn main() {
  use simple_logger::SimpleLogger;
  use std::collections::HashMap;
  #[cfg(target_os = "linux")]
  use std::path::Path;
  use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    menu::{MenuItem, MenuType},
    platform::system_tray::SystemTrayBuilder,
    window::Window,
  };
  SimpleLogger::new().init().unwrap();
  let event_loop = EventLoop::new();
  let mut windows = HashMap::new();

  let open_new_window = MenuItem::new("Open new window");
  let open_new_window_id = open_new_window.id();

  let focus_all_window = MenuItem::new("Focus window");
  let focus_all_window_id = focus_all_window.id();

  // Windows require Vec<u8> ICO file
  #[cfg(target_os = "windows")]
  let icon = include_bytes!("icon.ico").to_vec();
  // macOS require Vec<u8> PNG file
  #[cfg(target_os = "macos")]
  let icon = include_bytes!("icon.png").to_vec();
  // Linux require Pathbuf to PNG file
  #[cfg(target_os = "linux")]
  let icon = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/icon.png");

  // Only supported on macOS, linux and windows
  #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
  let _systemtray = SystemTrayBuilder::new(icon, vec![open_new_window, focus_all_window])
    .build(&event_loop)
    .unwrap();

  event_loop.run(move |event, event_loop, control_flow| {
    *control_flow = ControlFlow::Wait;

    match event {
      Event::WindowEvent { event, window_id } => {
        if event == WindowEvent::CloseRequested {
          println!("Window {:?} has received the signal to close", window_id);

          // Remove window from our hashmap
          windows.remove(&window_id);
        }
      }
      Event::MenuEvent {
        menu_id,
        origin: MenuType::SystemTray,
      } => {
        if menu_id == open_new_window_id {
          let window = Window::new(&event_loop).unwrap();
          windows.insert(window.id(), window);
        }
        if menu_id == focus_all_window_id {
          for window in windows.values() {
            window.set_focus();
          }
        }
        println!("Clicked on {:?}", menu_id);
      }
      _ => (),
    }
  });
}

#[cfg(any(target_os = "ios", target_os = "android",))]
fn main() {
  println!("This platform doesn't support run_return.");
}
