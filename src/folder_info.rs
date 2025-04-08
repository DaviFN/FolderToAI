use crate::file_info::FileInfo;
use crate::file_utils::{file_is_binary, get_file_size_in_bytes};

use crate::settings::Settings;
use std::fs;

#[derive(Debug)]
pub struct FolderInfo {
    pub folder_path: String,
    pub file_infos: Vec<FileInfo>,
    pub size_in_bytes: usize
}

impl FolderInfo {
    pub fn new(folder_path: &String, settings: &Settings) -> Result<Self, ()> {
        if let Ok(file_infos) = Self::obtain_file_infos(folder_path, settings, false) {
            let mut size_in_bytes: usize = 0;
            for file_info in &file_infos {
                size_in_bytes += file_info.size_in_bytes;
            }

            let folder_info: FolderInfo = FolderInfo{ folder_path: folder_path.to_string(), file_infos, size_in_bytes};
            return Ok(folder_info);
        }
        Err(())
    }

    pub fn get_number_of_files(&self) -> usize {
        self.file_infos.len()
    }

    pub fn get_number_of_files_whose_contents_should_be_loaded(&self) -> usize {
        let mut n: usize = 0;
        for file_info in &self.file_infos {
            if file_info.content_should_be_loaded() {
                n += 1;
            }
        }
        n
    }

    fn should_ignore_file(path: &str, settings: &Settings) -> bool
    {
        let path = std::path::Path::new(&path);

        for ancestor in path.ancestors() {
            if let Some(dir_name) = ancestor.file_name() {
                if let Some(dir_name_str) = dir_name.to_str() {
                    if settings.ignored_subfolders.contains(dir_name_str) {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn contains_at_least_one_file_that_should_not_be_ignored(&self) -> bool
    {
        for file_info in &self.file_infos {
            if !file_info.should_be_ignored {
                return true;
            }
        }
        false
    }

    pub fn obtain_file_infos(path: &str, settings: &Settings, recursive_call: bool) -> Result<Vec<FileInfo>, String>
    {
        let mut result: Vec<FileInfo> = Vec::new();

        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries {
                if let Ok(dir_entry) = entry {
                    if let Ok(file_type) = dir_entry.file_type() {
                        let is_file = file_type.is_file();
                        let is_directory = file_type.is_dir();
                        if let Some(path) = dir_entry.path().to_str() {
                            if is_file {
                                if let Ok(file_size) = get_file_size_in_bytes(path) {
                                    result.push(FileInfo::new(path.to_string(), file_size, Self::should_ignore_file(path, settings)));
                                }
                            }
                            else if is_directory {
                                if let Ok(file_infos_within_directory) = Self::obtain_file_infos(path, settings, true) {
                                    for file_info in file_infos_within_directory {
                                        result.push(file_info);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        else {
            return Err("could not obtain file infos".to_string())
        }

        if !recursive_call {
            for file_info in &mut result {
                file_info.filepath = file_info.filepath.replace(&format!("{}\\", path), "");
            }
        }

        Ok(result)
    }

    pub fn determine_binarity_of_next_file(&mut self, file_index: usize, settings: &Settings) {
        let file_info = &mut self.file_infos[file_index];
        if !Self::should_ignore_file(&file_info.filepath, settings) {
            file_info.is_binary = file_is_binary(file_info.filepath.as_str());
        }
    }

    pub fn determine_files_too_large(&mut self, max_file_size_in_bytes: usize, settings: &Settings) {
        for file_info in &mut self.file_infos {
            if !Self::should_ignore_file(&file_info.filepath, settings) {
                file_info.file_too_large = file_info.size_in_bytes > max_file_size_in_bytes;
            }
        }
    }

    pub fn load_next_file_content_if_required(&mut self, file_index: usize) -> bool
    {
        let file_info = &mut self.file_infos[file_index];
        if file_info.content_should_be_loaded() {
            if let Ok(file_bytes) = fs::read(&file_info.filepath) {
                file_info.file_content = Some(String::from_utf8_lossy(&file_bytes).into_owned());
            }
            return true;
        }
        false
    }

    pub fn number_of_files_that_could_not_be_loaded(&self) -> usize {
        let mut n: usize = 0;
        for file_info in &self.file_infos {
            if file_info.content_should_be_loaded() && !file_info.has_content_loaded() {
                n += 1;
            }
        }
        n
    }

    pub fn number_of_binary_files(&self) -> usize {
        let mut n: usize = 0;
        for file_info in &self.file_infos {
            if file_info.is_binary {
                n += 1;
            }
        }
        n
    }
}