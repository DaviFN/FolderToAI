use std::fmt;

#[derive(Debug)]
pub struct FileInfo
{
    pub filepath: String,
    pub size_in_bytes: usize,
    pub is_binary: bool,
    pub file_too_large: bool,
    pub should_be_ignored: bool,
    pub file_content: Option<String>
}

impl FileInfo {
    pub fn new(filepath: String, size_in_bytes: usize, should_be_ignored: bool) -> Self {
        FileInfo{ filepath, size_in_bytes, is_binary: false, file_too_large: false, should_be_ignored: should_be_ignored, file_content: None }
    }

    pub fn content_should_be_loaded(&self) -> bool {
        !self.is_binary && !self.file_too_large && !self.should_be_ignored
    }

    pub fn has_content_loaded(&self) -> bool {
        self.file_content.is_some()
    }
}

impl fmt::Display for FileInfo{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "file \"{}\", size in bytes: {}, contents:\n---\n{}\n---\n", self.filepath, self.size_in_bytes, self.file_content.clone().unwrap_or(String::from("no file content")))
    }
}