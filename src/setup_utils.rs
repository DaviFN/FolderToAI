
use crate::win_utils;

use std::fs::File;
use std::io::Write;
use std::path::Path;

const ICON_BINARY_DATA: &[u8] = include_bytes!("../mainicon.ico");

const DIRECTORY_REGISTRY_PATH: &str = "Software\\Classes\\Directory";

fn user_profile_path() -> String
{
    std::env::var("userprofile").unwrap_or("".to_string())
}

fn setup_path() -> String
{
    let user_profile_path = user_profile_path();
    if user_profile_path.is_empty() {
        return "".to_string();
    }
    user_profile_path + "\\FolderToAI"
}

fn setup_executable_path() -> String
{
    let setup_path = setup_path();
    if setup_path.is_empty() {
        return "".to_string();
    }
    setup_path + "\\FolderToAI.exe"
}

pub fn setup_icon_path() -> String
{
    let setup_path = setup_path();
    if setup_path.is_empty() {
        return "".to_string();
    }
    setup_path + "\\FolderToAI.ico"
}

pub fn settings_file_path() -> String
{
    let setup_path = setup_path();
    if setup_path.is_empty() {
        return "".to_string();
    }
    setup_path + "\\settings.json"
}

fn current_process_executable_path() -> String
{
    let mut path: String = String::from("");
    let current_exe = std::env::current_exe();
    if current_exe.is_ok() {
        path = String::from(current_exe.unwrap().to_str().unwrap_or(""));
    }
    path
}

pub fn is_being_executed_from_installation_location() -> bool
{
    Path::new(&current_process_executable_path()).starts_with(Path::new(&setup_path()))
}

fn create_icon_file_within_setup_path() -> bool
{
    if let Ok(mut file) = File::create(setup_icon_path()) {
        if let Ok(_) = file.write_all(ICON_BINARY_DATA) {
            return true;
        }
    }
    false
}

fn add_context_menu_on_windows_explorer_when_right_clicking_folder_background()
{
    win_utils::create_registry_key(&(String::from(DIRECTORY_REGISTRY_PATH) + "\\Background\\shell\\FolderToAI"), "", "FolderToAI");
    let folder_to_ai_value = format!("\"{}\"", setup_executable_path());
    win_utils::create_registry_key(&(String::from(DIRECTORY_REGISTRY_PATH) + "\\Background\\shell\\FolderToAI"), "Icon", &folder_to_ai_value);
    let folder_to_ai_command_value = format!("\"{}\" \"%V\"", setup_executable_path());
    win_utils::create_registry_key(&(String::from(DIRECTORY_REGISTRY_PATH) + "\\Background\\shell\\FolderToAI\\command"), "", &folder_to_ai_command_value);
}

fn add_context_menus_on_windows_explorer()
{
    add_context_menu_on_windows_explorer_when_right_clicking_folder_background();
}

fn remove_context_menu_from_windows_explorer_when_right_clicking_folder_background()
{
    win_utils::remove_registry_key(&(String::from(DIRECTORY_REGISTRY_PATH) + "\\Background\\shell\\FolderToAI"));
}

fn context_menu_registry_entry_exists() -> bool
{
    win_utils::registry_key_exists(&(String::from(DIRECTORY_REGISTRY_PATH) + "\\Background\\shell\\FolderToAI"))
}

fn assure_setup_directory_is_created() -> bool
{
    let setup_path = setup_path();
    if setup_path.is_empty() {
        return false;
    }
    if let Err(_) = std::fs::create_dir_all(setup_path) {
        return false;
    }
    true
}

fn remove_executable()
{
    std::fs::remove_file(setup_executable_path());
}

fn remove_icon()
{
    std::fs::remove_file(setup_icon_path());
}

fn remove_settings_file()
{
    std::fs::remove_file(settings_file_path());
}

fn remove_setup_directory()
{
    // note: remove_dir is used instead of remove_dir_all because supposedly the directory is empty
    // if it's not empty, we simply do not remove it, because, although unlikely, it's possible that the user has put relevant data inside it
    std::fs::remove_dir(setup_path());
}

fn assure_executable_exists_on_setup_path() -> bool
{
    let setup_executable_path = setup_executable_path();
    if setup_executable_path.is_empty() {
        return false;
    }
    if let Err(_) = std::fs::copy(current_process_executable_path(), setup_executable_path) {
        return false;
    }
    true
}

pub fn setup() -> bool
{
    if !assure_setup_directory_is_created() {
        return false;
    }
    if !assure_executable_exists_on_setup_path() {
        return false;
    }
    if !create_icon_file_within_setup_path() {
        return false;
    }
    add_context_menus_on_windows_explorer();
    true
}

pub fn is_installed() -> bool {
    if let Ok(executable_file_exists) = std::fs::exists(setup_executable_path()) {
        if !executable_file_exists {
            return false;
        }
    }
    else {
        return false;
    }

    if !context_menu_registry_entry_exists() {
        return false;
    }

    true
}

pub fn is_completely_uninstalled() -> bool {
    if context_menu_registry_entry_exists() {
        return false;
    }
    if let Ok(setup_directory_exists) = std::fs::exists(setup_path()) {
        if setup_directory_exists {
            return false;
        }
    }
    else {
        return false;
    }

    true
}

pub fn uninstall() -> bool
{
    remove_executable();
    remove_icon();
    remove_settings_file();
    remove_setup_directory();
    remove_context_menu_from_windows_explorer_when_right_clicking_folder_background();
    true
}