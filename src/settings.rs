use serde::{Serialize, Deserialize};
use size::Size;
use std::collections::BTreeSet;
use std::io::Write;

#[derive(Serialize, Deserialize)]
struct Vec2Serializable {
    x: f32,
    y: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    window_size: Vec2Serializable,
    pub pasting_with_ctrlv_advances_to_next_message: bool,
    pub file_size_limit: usize,
    #[serde(skip)]
    ignored_subfolders_input: String,
    pub ignored_subfolders: BTreeSet<String>
}

impl Settings {
    pub fn new() -> Self {
        const DEFAULT_WINDOW_WIDTH: f32 = 510.0;
        const DEFAULT_WINDOW_HEIGHT: f32 = 400.0;
        const DEFAULT_FILE_SIZE_LIMIT: usize = 100 * 1024; // 100 KiB

        let mut settings = Settings{window_size: Vec2Serializable{x: DEFAULT_WINDOW_WIDTH, y: DEFAULT_WINDOW_HEIGHT}, pasting_with_ctrlv_advances_to_next_message: true, ignored_subfolders_input: String::from(""), ignored_subfolders: BTreeSet::new(), file_size_limit: DEFAULT_FILE_SIZE_LIMIT};

        settings.initialize_default_ignored_subfolders();

        settings
    }

    pub fn window_size(&self) -> egui::Vec2
    {
        egui::Vec2{x: self.window_size.x, y: self.window_size.y}
    }

    pub fn set_window_size(&mut self, window_size: &egui::Vec2)
    {
        self.window_size.x = window_size.x;
        self.window_size.y = window_size.y;
    }


    fn initialize_default_ignored_subfolders(&mut self)
    {
        let default_ignored_subfolders = vec![
            ".cache",
            ".cargo",
            ".git",
            ".gradle",
            ".idea",
            ".mvn",
            ".npm",
            ".pytest_cache",
            ".rustup",
            ".svn",
            ".venv",
            ".vs",
            ".vscode",
            "bin",
            "build",
            "dist",
            "node_modules",
            "obj",
            "target",
            "tmp",
            "venv",
            "__pycache__"
        ];

        for subfolder in default_ignored_subfolders {
            self.ignored_subfolders.insert(subfolder.to_string());
        }
    }

    pub fn save_to_file(&self, path: &str) -> bool {
        if let Ok(json) = serde_json::to_string_pretty(&self) {
            if let Ok(mut file) = std::fs::File::create(path) {
                file.write_all(json.as_bytes());
                return true;
            }
        }
        false
    }

    pub fn load_from_file(&mut self, path: &str) -> bool {
        if let Ok(json) = std::fs::read_to_string(path) {
            if let Ok(loaded_settings) = serde_json::from_str(&json) {
                *self = loaded_settings;
                return true;
            }
        }
        false
    }

    fn show_file_size_limit_settings_gui(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("Current file size limit: {}", Size::from_bytes(self.file_size_limit)));

        const RANGE_MIN: usize = 100; // 100 bytes
        const RANGE_MAX: usize = 1024 * 1024 * 1024; // 1 GiB
        ui.add(egui::Slider::new(&mut self.file_size_limit, RANGE_MIN..=RANGE_MAX).text("File size limit (bytes)"));
    }

    fn show_folder_ignoring_settings_gui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Add subfolder to ignore:");
            let text_edit_response = ui.add(egui::TextEdit::singleline(&mut self.ignored_subfolders_input));
    
            if text_edit_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if self.ignored_subfolders_input_contains_valid_folder_name() {
                    self.add_subfolder_to_ignore();
                }
                text_edit_response.request_focus();
            }

            if ui.button("Add").clicked() {
                if self.ignored_subfolders_input_contains_valid_folder_name() {
                    self.add_subfolder_to_ignore();
                }
            }
        });

        ui.separator();
        ui.label("Currently ignoring subfolders:");
        
        let mut subfolders_to_remove = Vec::new();
        let max_items_per_line = 5;
        
        let subfolders: Vec<_> = self.ignored_subfolders.iter().cloned().collect();
        
        for chunk in subfolders.chunks(max_items_per_line) {
            ui.horizontal(|ui| {
                for subfolder in chunk {
                    ui.label(subfolder);
                    if ui.button("❌").clicked() {
                        subfolders_to_remove.push(subfolder.clone());
                    }
                    ui.add_space(2.0);
                }
            });
            ui.add_space(2.0);
        }
        
        for subfolder in subfolders_to_remove {
            self.remove_subfolder_to_ignore(&subfolder);
        }
    }



    pub fn show_gui(&mut self, ui: &mut egui::Ui) {
        self.show_file_size_limit_settings_gui(ui);
        ui.separator();
        self.show_folder_ignoring_settings_gui(ui);
    }

    fn add_subfolder_to_ignore(&mut self) {
        if !self.ignored_subfolders_input.is_empty() {
            self.ignored_subfolders.insert(self.ignored_subfolders_input.clone());
            self.ignored_subfolders_input.clear();
        }
    }

    fn remove_subfolder_to_ignore(&mut self, folder: &str) {
        self.ignored_subfolders.remove(folder);
    }

    fn ignored_subfolders_input_contains_valid_folder_name(&self) -> bool
    {
        Self::is_valid_folder_name(&self.ignored_subfolders_input)
    }

    fn is_valid_folder_name(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        let forbidden_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
        if name.chars().any(|c| forbidden_chars.contains(&c)) {
            return false;
        }

        if name.ends_with(' ') || name.ends_with('.') {
            return false;
        }

        true
    }
}