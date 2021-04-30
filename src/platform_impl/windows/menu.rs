use raw_window_handle::RawWindowHandle;
use std::os::windows::ffi::OsStrExt;
use winapi::{
    ctypes::c_void,
    shared::{
        basetsd,
        guiddef::REFIID,
        minwindef,
        minwindef::{DWORD, UINT, ULONG},
        windef,
        windef::{HWND, POINTL},
        winerror::S_OK,
    },
    um::{
        commctrl,
        objidl::IDataObject,
        oleidl::{IDropTarget, IDropTargetVtbl, DROPEFFECT_COPY, DROPEFFECT_NONE},
        shellapi, unknwnbase,
        winnt::HRESULT,
        winuser,
    },
};

use std::{
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::menu::{Menu, MenuItem};
use crate::{event::Event, window::WindowId as SuperWindowId};

pub struct MenuHandler {
    window: HWND,
    send_event: Box<dyn Fn(Event<'static, ()>)>,
}

#[allow(non_snake_case)]
impl MenuHandler {
    pub fn new(window: HWND, send_event: Box<dyn Fn(Event<'static, ()>)>) -> MenuHandler {
        let data = Box::new(MenuHandlerData { window, send_event });
        MenuHandler { window, send_event }
    }
    fn send_event(&self, event: Event<'static, ()>) {
        (self.send_event)(event);
    }
}

pub fn initialize(menu: Vec<Menu>, window_handle: RawWindowHandle, menu_handler: MenuHandler) {
    dbg!(menu);

    if let RawWindowHandle::Windows(handle) = window_handle {
        let sender: *mut MenuHandler = Box::into_raw(Box::new(menu_handler));

        unsafe {
            commctrl::SetWindowSubclass(
                handle.hwnd as *mut _,
                Some(subclass_proc),
                0,
                sender as basetsd::DWORD_PTR,
            );

            let testing_menu = winuser::CreateMenu();
            let subitem = winuser::MENUITEMINFOW {
                cbSize: std::mem::size_of::<winuser::MENUITEMINFOW>() as u32,
                fMask: winuser::MIIM_STRING | winuser::MIIM_ID,
                fType: winuser::MFT_STRING,
                fState: winuser::MFS_ENABLED,
                // Received on low-word of wParam when WM_COMMAND
                // It could represent the menu ID
                wID: 3653,
                hSubMenu: std::ptr::null_mut(),
                hbmpChecked: std::ptr::null_mut(),
                hbmpUnchecked: std::ptr::null_mut(),
                dwItemData: 0,
                dwTypeData: to_wstring("&Close\tAlt+C").as_mut_ptr(),
                cch: 5,
                hbmpItem: std::ptr::null_mut(),
            };
            winuser::InsertMenuItemW(testing_menu, 0, 0, &subitem as *const _);

            let system_menu = winuser::CreateMenu();
            let item = winuser::MENUITEMINFOW {
                cbSize: std::mem::size_of::<winuser::MENUITEMINFOW>() as u32,
                fMask: winuser::MIIM_STRING | winuser::MIIM_SUBMENU,
                fType: winuser::MFT_STRING,
                fState: winuser::MFS_ENABLED,
                wID: 0,
                hSubMenu: testing_menu,
                hbmpChecked: std::ptr::null_mut(),
                hbmpUnchecked: std::ptr::null_mut(),
                dwItemData: 0,
                dwTypeData: to_wstring("Outer").as_mut_ptr(),
                cch: 5,
                hbmpItem: std::ptr::null_mut(),
            };
            winuser::InsertMenuItemW(system_menu, 0, 0, &item as *const _);

            winuser::SetMenu(handle.hwnd as *mut _, system_menu);
        }
    }
}

fn to_wstring(str: &str) -> Vec<u16> {
    let v: Vec<u16> = std::ffi::OsStr::new(str)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect();
    v
}

unsafe extern "system" fn subclass_proc(
    hwnd: windef::HWND,
    u_msg: minwindef::UINT,
    w_param: minwindef::WPARAM,
    l_param: minwindef::LPARAM,
    _id: basetsd::UINT_PTR,
    data: basetsd::DWORD_PTR,
) -> minwindef::LRESULT {
    match u_msg {
        winuser::WM_COMMAND => {
            let proxy = &mut *(data as *mut MenuHandler);
            let lo_word = minwindef::LOWORD(w_param as u32);
            proxy.send_event(Event::MenuEvent(lo_word.to_string()));
            0
        }
        winuser::WM_DESTROY => {
            Box::from_raw(data as *mut MenuHandler);
            0
        }
        _ => commctrl::DefSubclassProc(hwnd, u_msg, w_param, l_param),
    }
}
