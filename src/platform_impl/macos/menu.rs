// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0

use cocoa::{
  appkit::{NSApp, NSApplication, NSEventModifierFlags, NSMenu, NSMenuItem},
  base::{id, nil, selector},
  foundation::{NSAutoreleasePool, NSString},
};
use objc::{
  declare::ClassDecl,
  rc::autoreleasepool,
  runtime::{Class, Object, Sel},
};
use std::sync::Once;

use crate::{
  event::Event,
  menu::{Menu, MenuId, MenuItem, MenuType},
};

use super::{app_state::AppState, event::EventWrapper};

static BLOCK_PTR: &str = "taoMenuItemBlockPtr";

pub(crate) struct KeyEquivalent<'a> {
  pub(crate) key: &'a str,
  pub(crate) masks: Option<NSEventModifierFlags>,
}

#[derive(Debug)]
struct Action(Box<u32>);

pub fn initialize(menu: Vec<Menu>) {
  autoreleasepool(|| unsafe {
    let menubar = NSMenu::new(nil).autorelease();

    for menu in menu {
      // create our menu
      let menu_item = NSMenuItem::new(nil).autorelease();
      menubar.addItem_(menu_item);
      // prepare our submenu tree
      let menu_title = NSString::alloc(nil).init_str(&menu.title);
      let menu_object = NSMenu::alloc(nil).initWithTitle_(menu_title).autorelease();

      // create menu
      for item in &menu.items {
        let item_obj: *mut Object = match item {
          // Custom menu
          MenuItem::Custom(custom_menu) => {
            // build accelerators if provided
            let mut key_equivalent = None;
            let mut accelerator_string: String;
            if let Some(accelerator) = &custom_menu.keyboard_accelerators {
              accelerator_string = accelerator.clone();
              let mut ns_modifier_flags: NSEventModifierFlags = NSEventModifierFlags::empty();

              if accelerator_string.contains("<Primary>") {
                accelerator_string = accelerator_string.replace("<Primary>", "");
                ns_modifier_flags.insert(NSEventModifierFlags::NSCommandKeyMask);
              }

              if accelerator_string.contains("<Shift>") {
                accelerator_string = accelerator_string.replace("<Shift>", "");
                ns_modifier_flags.insert(NSEventModifierFlags::NSShiftKeyMask);
              }

              if accelerator_string.contains("<Ctrl>") {
                accelerator_string = accelerator_string.replace("<Ctrl>", "");
                ns_modifier_flags.insert(NSEventModifierFlags::NSControlKeyMask);
              }

              let mut masks = None;
              if !ns_modifier_flags.is_empty() {
                masks = Some(ns_modifier_flags);
              }

              key_equivalent = Some(KeyEquivalent {
                key: accelerator_string.as_str(),
                masks,
              });
            }

            make_custom_menu_item(
              custom_menu.id,
              &custom_menu.name,
              None,
              key_equivalent,
              MenuType::Menubar,
            )
          }
          // Separator
          MenuItem::Separator => NSMenuItem::separatorItem(nil),
          // About
          MenuItem::About(app_name) => {
            let title = format!("About {}", app_name);
            make_menu_item(
              title.as_str(),
              Some(selector("orderFrontStandardAboutPanel:")),
              None,
              MenuType::Menubar,
            )
          }
          // Close window
          MenuItem::CloseWindow => make_menu_item(
            "Close Window",
            Some(selector("performClose:")),
            Some(KeyEquivalent {
              key: "w",
              masks: None,
            }),
            MenuType::Menubar,
          ),
          MenuItem::Quit => make_menu_item(
            "Quit",
            Some(selector("terminate:")),
            Some(KeyEquivalent {
              key: "q",
              masks: None,
            }),
            MenuType::Menubar,
          ),
          MenuItem::Hide => make_menu_item(
            "Hide",
            Some(selector("hide:")),
            Some(KeyEquivalent {
              key: "h",
              masks: None,
            }),
            MenuType::Menubar,
          ),
          MenuItem::HideOthers => make_menu_item(
            "Hide Others",
            Some(selector("hideOtherApplications:")),
            Some(KeyEquivalent {
              key: "h",
              masks: Some(
                NSEventModifierFlags::NSAlternateKeyMask | NSEventModifierFlags::NSCommandKeyMask,
              ),
            }),
            MenuType::Menubar,
          ),
          MenuItem::ShowAll => make_menu_item(
            "Show All",
            Some(selector("unhideAllApplications:")),
            None,
            MenuType::Menubar,
          ),
          MenuItem::EnterFullScreen => make_menu_item(
            "Enter Full Screen",
            Some(selector("toggleFullScreen:")),
            Some(KeyEquivalent {
              key: "f",
              masks: Some(
                NSEventModifierFlags::NSCommandKeyMask | NSEventModifierFlags::NSControlKeyMask,
              ),
            }),
            MenuType::Menubar,
          ),
          MenuItem::Minimize => make_menu_item(
            "Minimize",
            Some(selector("performMiniaturize:")),
            Some(KeyEquivalent {
              key: "m",
              masks: None,
            }),
            MenuType::Menubar,
          ),
          MenuItem::Zoom => make_menu_item(
            "Zoom",
            Some(selector("performZoom:")),
            None,
            MenuType::Menubar,
          ),
          MenuItem::Copy => make_menu_item(
            "Copy",
            Some(selector("copy:")),
            Some(KeyEquivalent {
              key: "c",
              masks: None,
            }),
            MenuType::Menubar,
          ),
          MenuItem::Cut => make_menu_item(
            "Cut",
            Some(selector("cut:")),
            Some(KeyEquivalent {
              key: "x",
              masks: None,
            }),
            MenuType::Menubar,
          ),
          MenuItem::Paste => make_menu_item(
            "Paste",
            Some(selector("paste:")),
            Some(KeyEquivalent {
              key: "v",
              masks: None,
            }),
            MenuType::Menubar,
          ),
          MenuItem::Undo => make_menu_item(
            "Undo",
            Some(selector("undo:")),
            Some(KeyEquivalent {
              key: "z",
              masks: None,
            }),
            MenuType::Menubar,
          ),
          MenuItem::Redo => make_menu_item(
            "Redo",
            Some(selector("redo:")),
            Some(KeyEquivalent {
              key: "Z",
              masks: None,
            }),
            MenuType::Menubar,
          ),
          MenuItem::SelectAll => make_menu_item(
            "Select All",
            Some(selector("selectAll:")),
            Some(KeyEquivalent {
              key: "a",
              masks: None,
            }),
            MenuType::Menubar,
          ),
          MenuItem::Services => {
            let item = make_menu_item("Services", None, None, MenuType::Menubar);
            let app_class = class!(NSApplication);
            let app: id = msg_send![app_class, sharedApplication];
            let services: id = msg_send![app, servicesMenu];
            let _: () = msg_send![&*item, setSubmenu: services];
            item
          }
        };

        menu_object.addItem_(item_obj);
      }

      menu_item.setSubmenu_(menu_object);
    }

    // Set the menu as main menu for the app
    let app = NSApp();
    app.setMainMenu_(menubar);
  });
}

fn make_menu_alloc() -> *mut Object {
  unsafe { msg_send![make_menu_item_class(), alloc] }
}

pub(crate) fn make_custom_menu_item(
  id: MenuId,
  title: &str,
  selector: Option<Sel>,
  key_equivalent: Option<KeyEquivalent<'_>>,
  menu_type: MenuType,
) -> *mut Object {
  let alloc = make_menu_alloc();
  let menu_id = Box::new(Action(Box::new(id.0)));
  let ptr = Box::into_raw(menu_id);

  unsafe {
    (&mut *alloc).set_ivar(BLOCK_PTR, ptr as usize);
    let _: () = msg_send![&*alloc, setTarget:&*alloc];
    let title = NSString::alloc(nil).init_str(title);
    make_menu_item_from_alloc(alloc, title, selector, key_equivalent, menu_type)
  }
}

pub(crate) fn make_menu_item(
  title: &str,
  selector: Option<Sel>,
  key_equivalent: Option<KeyEquivalent<'_>>,
  menu_type: MenuType,
) -> *mut Object {
  let alloc = make_menu_alloc();
  unsafe {
    let title = NSString::alloc(nil).init_str(title);
    make_menu_item_from_alloc(alloc, title, selector, key_equivalent, menu_type)
  }
}

fn make_menu_item_from_alloc(
  alloc: *mut Object,
  title: *mut Object,
  selector: Option<Sel>,
  key_equivalent: Option<KeyEquivalent<'_>>,
  menu_type: MenuType,
) -> *mut Object {
  unsafe {
    let (key, masks) = match key_equivalent {
      Some(ke) => (NSString::alloc(nil).init_str(ke.key), ke.masks),
      None => (NSString::alloc(nil).init_str(""), None),
    };
    // if no selector defined, that mean it's a custom
    // menu so fire our handler
    let selector = match selector {
      Some(selector) => selector,
      None => match menu_type {
        MenuType::Menubar => sel!(fireMenubarAction:),
        MenuType::SystemTray => sel!(fireStatusbarAction:),
      },
    };

    // allocate our item to our class
    let item: id = msg_send![alloc, initWithTitle:&*title action:selector keyEquivalent:&*key];
    if let Some(masks) = masks {
      item.setKeyEquivalentModifierMask_(masks)
    }

    item
  }
}

fn make_menu_item_class() -> *const Class {
  static mut APP_CLASS: *const Class = 0 as *const Class;
  static INIT: Once = Once::new();

  INIT.call_once(|| unsafe {
    let superclass = class!(NSMenuItem);
    let mut decl = ClassDecl::new("TaoMenuItem", superclass).unwrap();
    decl.add_ivar::<usize>(BLOCK_PTR);

    decl.add_method(
      sel!(dealloc),
      dealloc_custom_menuitem as extern "C" fn(&Object, _),
    );

    decl.add_method(
      sel!(fireMenubarAction:),
      fire_menu_bar_click as extern "C" fn(&Object, _, id),
    );

    decl.add_method(
      sel!(fireStatusbarAction:),
      fire_status_bar_click as extern "C" fn(&Object, _, id),
    );

    APP_CLASS = decl.register();
  });

  unsafe { APP_CLASS }
}

extern "C" fn fire_status_bar_click(this: &Object, _: Sel, _item: id) {
  send_event(this, MenuType::SystemTray);
}

extern "C" fn fire_menu_bar_click(this: &Object, _: Sel, _item: id) {
  send_event(this, MenuType::Menubar);
}

fn send_event(this: &Object, origin: MenuType) {
  let menu_id = unsafe {
    let ptr: usize = *this.get_ivar(BLOCK_PTR);
    let obj = ptr as *const Action;
    &*obj
  };
  let event = Event::MenuEvent {
    menu_id: MenuId(*menu_id.0),
    origin,
  };
  AppState::queue_event(EventWrapper::StaticEvent(event));
}

extern "C" fn dealloc_custom_menuitem(this: &Object, _: Sel) {
  unsafe {
    let ptr: usize = *this.get_ivar(BLOCK_PTR);
    let obj = ptr as *mut Action;
    if !obj.is_null() {
      let _handler = Box::from_raw(obj);
    }
    let _: () = msg_send![super(this, class!(NSMenuItem)), dealloc];
  }
}
