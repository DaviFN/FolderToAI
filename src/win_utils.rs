use std::ffi::CString;
use std::io;
use winapi::um::winuser::{IDYES, MB_OK, MB_YESNO, MessageBoxA};
use winreg::enums::*;
use winreg::RegKey;

pub fn pause_cmd()
{
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
}

pub fn message_box(message: &str, title: &str)
{
    let lp_text = CString::new(message).unwrap();
    let lp_caption = CString::new(title).unwrap();
    unsafe {
        MessageBoxA(
            std::ptr::null_mut(),
            lp_text.as_ptr(),
            lp_caption.as_ptr(),
            MB_OK,
        );
    }
}

pub fn yesno_message_box(message: &str, title: &str) -> bool
{
    let lp_text = CString::new(message).unwrap();
    let lp_caption = CString::new(title).unwrap();

    let mut result = false;
    unsafe {
        result = MessageBoxA(
            std::ptr::null_mut(),
            lp_text.as_ptr(),
            lp_caption.as_ptr(),
            MB_YESNO,
        ) == IDYES;
    }
    result
}

pub fn create_registry_key(registry_path: &str, key_name: &str, key_value: &str) -> Result<(), ()>
{
    if let Ok((key, _)) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(registry_path) {
        if let Ok(_) = key.set_value(key_name, &key_value) {
            return Ok(());
        }
    }
    Err(())
}

pub fn remove_registry_key(registry_path: &str) -> Result<(), ()>
{
    if let Ok(_) = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(registry_path) {
        return Ok(())
    }
    Err(())
}

pub fn registry_key_exists(registry_path: &str) -> bool
{
    if let Ok(_) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(registry_path) {
        return true;
    }
    false
}