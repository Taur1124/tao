use std::mem::MaybeUninit;

use webview2_com_sys::Windows::Win32::{
  Foundation::{HWND, LPARAM, WPARAM},
  UI::WindowsAndMessaging::*,
};

use crate::platform_impl::platform::event_loop::ProcResult;

pub fn is_msg_ime_related(msg_kind: u32) -> bool {
  match msg_kind {
    WM_IME_COMPOSITION
    | WM_IME_COMPOSITIONFULL
    | WM_IME_STARTCOMPOSITION
    | WM_IME_ENDCOMPOSITION
    | WM_IME_CHAR
    | WM_CHAR
    | WM_SYSCHAR => true,
    _ => false,
  }
}

pub struct MinimalIme {
  // True if we're currently receiving messages belonging to a finished IME session.
  getting_ime_text: bool,

  utf16parts: Vec<u16>,
}
impl Default for MinimalIme {
  fn default() -> Self {
    MinimalIme {
      getting_ime_text: false,
      utf16parts: Vec::with_capacity(16),
    }
  }
}
impl MinimalIme {
  pub(crate) fn process_message(
    &mut self,
    hwnd: HWND,
    msg_kind: u32,
    wparam: WPARAM,
    _lparam: LPARAM,
    result: &mut ProcResult,
  ) -> Option<String> {
    match msg_kind {
      WM_IME_ENDCOMPOSITION => {
        self.getting_ime_text = true;
      }
      WM_CHAR | WM_SYSCHAR => {
        if self.getting_ime_text {
          *result = ProcResult::Value(0);
          self.utf16parts.push(wparam.0 as u16);

          let more_char_coming;
          unsafe {
            let mut next_msg = MaybeUninit::uninit();
            let has_message = PeekMessageW(
              next_msg.as_mut_ptr(),
              hwnd,
              WM_KEYFIRST,
              WM_KEYLAST,
              PM_NOREMOVE,
            );
            let has_message = has_message.as_bool();
            if !has_message {
              more_char_coming = false;
            } else {
              let next_msg = next_msg.assume_init().message;
              if next_msg == WM_CHAR || next_msg == WM_SYSCHAR {
                more_char_coming = true;
              } else {
                more_char_coming = false;
              }
            }
          }
          if !more_char_coming {
            let result = String::from_utf16(&self.utf16parts).ok();
            self.utf16parts.clear();
            self.getting_ime_text = false;
            return result;
          }
        }
      }
      _ => (),
    }

    None
  }
}
