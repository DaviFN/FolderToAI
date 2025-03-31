#![windows_subsystem = "windows"]

mod clipboard_utils;
mod core_utils;
mod file_info;
mod file_utils;
mod folder_info;
mod input_utils;
mod settings;
mod setup_utils;
mod win_utils;

use egui::{RichText, Color32};
use folder_info::FolderInfo;
use input_utils::InputManager;
use settings::Settings;
use size::Size;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::thread;
use std::time::{Duration, Instant};

const APP_NAME: &str = "FolderToAI";
const LINK_TO_GIT_REPO: &str = "https://github.com/DaviFN/FolderToAI";

const GUI_UPDATE_DELAY_MS: u128 = 50;
const GUI_SIZE_OF_SPACE_AFTER_SEPARATOR: f32 = 5.0;

#[derive(PartialEq)]
enum FolderToAiState {
    Initializing,
    ObtainingInitialInformationAboutTheFiles,
    DeterminingBinaryFiles,
    LoadingContents,
    ProcessingContents,
    ReadyForUse,
    Error
}

#[derive(PartialEq)]
#[derive(Clone)]
enum FolderToAiUserInput {
    None,
    StepForwardInMessages,
    StepBackwardInMessages
}
struct FolderToAiApp {
    folder_path: String,
    state: FolderToAiState,
    gui_has_ever_been_updated: bool,
    previous_window_size: egui::Vec2,
    folder_info: Option<FolderInfo>,
    total_n_files: usize,
    n_binary_files: usize,
    total_n_files_to_load: usize,
    n_files_already_determined_whether_binary_or_not: usize,
    progress_of_determining_binary_files: f64,
    n_files_already_loaded_if_required_loading: usize,
    n_files_loaded: usize,
    n_files_that_could_not_be_loaded: usize,
    progress_of_loading_contents: f64,
    folder_representation_messages: Vec<String>,
    current_selected_message_index: usize,
    input_manager: InputManager,
    current_user_input: FolderToAiUserInput,
    clipboard_content_information_message: String,
    settings: Settings
}

impl FolderToAiApp {
    fn new(folder_path: String) -> Self
    {
        FolderToAiApp {
            folder_path: folder_path,
            state: FolderToAiState::Initializing,
            gui_has_ever_been_updated: false,
            previous_window_size: egui::Vec2{x: 0.0, y: 0.0},
            folder_info: None,
            total_n_files: 0,
            n_binary_files: 0,
            total_n_files_to_load: 0,
            n_files_already_determined_whether_binary_or_not: 0,
            progress_of_determining_binary_files: 0.0,
            n_files_already_loaded_if_required_loading: 0,
            n_files_loaded: 0,
            n_files_that_could_not_be_loaded: 0,
            progress_of_loading_contents: 0.0,
            folder_representation_messages: Vec::new(),
            current_selected_message_index: 0,
            input_manager: InputManager::new(),
            current_user_input: FolderToAiUserInput::None,
            clipboard_content_information_message: String::from("Messages being created..."),
            settings: Settings::new()
        }
    }

    fn on_first_gui_update(&mut self, ctx: &eframe::egui::Context)
    {
        ctx.set_theme(egui::Theme::Dark);

        self.settings.load_from_file(&setup_utils::settings_file_path());

        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(self.settings.window_size()));
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
    }

    fn on_window_resize(&mut self, new_size: &egui::Vec2)
    {
        if self.gui_has_ever_been_updated {
            self.settings.set_window_size(new_size);
        }
    }

    fn step_forward_current_message_and_send_to_clipboard_if_possible(&mut self)
    {
        if self.current_selected_message_index + 1 < self.folder_representation_messages.len() {
            self.current_selected_message_index += 1;
            clipboard_utils::set_clipboard_content(&self.folder_representation_messages[self.current_selected_message_index]);
            self.clipboard_content_information_message = format!("Clipboard has been set to message {} of {}", self.current_selected_message_index + 1, self.folder_representation_messages.len());
        }
    }

    fn step_backwards_current_message_and_send_to_clipboard_if_possible(&mut self)
    {
        if self.current_selected_message_index != 0 {
            self.current_selected_message_index -= 1;
            clipboard_utils::set_clipboard_content(&self.folder_representation_messages[self.current_selected_message_index]);
            self.clipboard_content_information_message = format!("Clipboard has been set to message {} of {}", self.current_selected_message_index + 1, self.folder_representation_messages.len());
        }
    }

    fn key_combination_to_step_forward_in_messages_is_pressed(&mut self) -> bool
    {
        self.input_manager.is_right_key_pressed() || self.input_manager.is_control_v_pressed()
    }

    fn key_combination_to_step_backwards_in_messages_is_pressed(&mut self) -> bool
    {
        self.input_manager.is_left_key_pressed()
    }

    fn get_user_input(&mut self) -> FolderToAiUserInput
    {
        if self.key_combination_to_step_forward_in_messages_is_pressed() {
            return FolderToAiUserInput::StepForwardInMessages;
        }
        if self.key_combination_to_step_backwards_in_messages_is_pressed() {
            return FolderToAiUserInput::StepBackwardInMessages;
        }
        FolderToAiUserInput::None
    }

    fn deal_with_user_input(&mut self)
    {
        let previous_user_input = self.current_user_input.clone();
        self.current_user_input = self.get_user_input();

        if self.folder_representation_messages.is_empty() {
            return;
        }

        if previous_user_input == FolderToAiUserInput::None {
            match self.current_user_input {
                FolderToAiUserInput::StepForwardInMessages => {
                    thread::sleep(Duration::from_millis(200)); // some chats may be a bit slow, this delay is added to account for that
                    self.step_forward_current_message_and_send_to_clipboard_if_possible();
                },
                FolderToAiUserInput::StepBackwardInMessages => {
                    thread::sleep(Duration::from_millis(200)); // some chats may be a bit slow, this delay is added to account for that
                    self.step_backwards_current_message_and_send_to_clipboard_if_possible();
                }
                _ => {}
            }
        }
    }

    fn show_ui_heading(ui: &mut egui::Ui, text: &str)
    {
        let heading = RichText::new(text).color(Color32::LIGHT_GRAY).size(20.0);
        ui.label(heading);
    }

    fn current_state_info_string(&self) -> String {
        match self.state {
            FolderToAiState::Initializing | FolderToAiState::ObtainingInitialInformationAboutTheFiles => {
                String::from("Obtaining initial info...")
            },
            FolderToAiState::DeterminingBinaryFiles => {
                format!("Determining binary files... ({}/{})", self.n_files_already_determined_whether_binary_or_not, self.total_n_files)
            },
            FolderToAiState::LoadingContents => {
                format!("Loading contents... ({}/{})", self.n_files_loaded, self.total_n_files_to_load)
            },
            _ => {
                if self.state != FolderToAiState::ReadyForUse {
                    String::from("Loading process in execution, please wait...")
                }
                else {
                    let all_files_have_been_loaded = self.n_files_that_could_not_be_loaded == 0;
                    let n_files_loaded = self.total_n_files_to_load - self.n_files_that_could_not_be_loaded;
                    if all_files_have_been_loaded {
                        format!("All relevant files have been successfully loaded ({}/{})", n_files_loaded, self.total_n_files_to_load)
                    }
                    else {
                        format!("Not all relevant files could be loaded ({}/{}); perhaps they're being used somehow?", n_files_loaded, self.total_n_files_to_load)
                    }
                }
            }
        }
    }

    fn current_state_string_info_color(&self) -> egui::Color32
    {
        if self.state == FolderToAiState::ReadyForUse { 
            let all_files_have_been_loaded = self.n_files_that_could_not_be_loaded == 0;
            if all_files_have_been_loaded {
                return egui::Color32::GREEN;
            }
            return egui::Color32::RED;
        }
        egui::Color32::LIGHT_GRAY
    }

    fn show_loading_process_bar(&self, ui: &mut egui::Ui)
    {
        let mut loading_process_progress = 0.0;
        if self.state != FolderToAiState::Initializing && self.state != FolderToAiState::ObtainingInitialInformationAboutTheFiles {
            let total_n_steps_in_loading_process = 2 * self.total_n_files;
            let current_step_in_loading_process = self.n_files_already_determined_whether_binary_or_not + self.n_files_already_loaded_if_required_loading;
            
            if total_n_steps_in_loading_process > 0 {
                loading_process_progress = current_step_in_loading_process as f64 / total_n_steps_in_loading_process as f64;
            }
            else {
                loading_process_progress = 1.0;
            }
        }

        let loading_process_progress_bar = egui::ProgressBar::new(loading_process_progress as f32).show_percentage();
        ui.add(loading_process_progress_bar);
    }

    fn show_loading_progress_gui(&mut self, ui: &mut egui::Ui)
    {
        let loading_progress_header_string = match self.state {
            FolderToAiState::ReadyForUse => {
                "Loading process (completed)"
            }
            _ => {
                "Loading process (in progress)"
            }
        };
        
        Self::show_ui_heading(ui, loading_progress_header_string);
        ui.add_space(GUI_SIZE_OF_SPACE_AFTER_SEPARATOR);

        ui.label(egui::RichText::new(self.current_state_info_string()).color(self.current_state_string_info_color()));
        
        self.show_loading_process_bar(ui);
    }

    fn show_folder_information_gui(&mut self, ui: &mut egui::Ui) {
        Self::show_ui_heading(ui, "Folder information");
        ui.add_space(GUI_SIZE_OF_SPACE_AFTER_SEPARATOR);

        ui.label(format!("Folder: \"{}\"", self.folder_path));
        
        let mut size_info_str = String::from("Size: ");
        if self.state == FolderToAiState::Initializing || self.state == FolderToAiState::ObtainingInitialInformationAboutTheFiles {
            size_info_str += "being determined...";
        }
        else {
            let size_in_bytes = self.folder_info.as_mut().unwrap().size_in_bytes;
            size_info_str += &format!("{}", Size::from_bytes(size_in_bytes));
        }
        ui.label(size_info_str);

        let mut n_files_info_str = String::from("Total number of files: ");
        if self.state == FolderToAiState::Initializing || self.state == FolderToAiState::ObtainingInitialInformationAboutTheFiles {
            n_files_info_str += "being determined...";
        }
        else {
            n_files_info_str += &format!("{}", self.folder_info.as_mut().unwrap().get_number_of_files());
        }
        ui.label(n_files_info_str);

        let mut size_info_str = String::from("Number of binary files: ");
        if self.state == FolderToAiState::Initializing || self.state == FolderToAiState::ObtainingInitialInformationAboutTheFiles || self.state == FolderToAiState::DeterminingBinaryFiles {
            size_info_str += "being determined...";
        }
        else {
            let percentage_of_binary_files: f64 = self.n_binary_files as f64 / self.total_n_files as f64;
            size_info_str += &format!("{} ({}% of total)", self.n_binary_files, format!("{:.2}", percentage_of_binary_files * 100.0));
        }
        ui.label(size_info_str);
    }

    fn show_messages_gui(&mut self, ui: &mut egui::Ui)
    {
        Self::show_ui_heading(ui, "Messages");
        ui.add_space(GUI_SIZE_OF_SPACE_AFTER_SEPARATOR);

        ui.label("Send the messages into an AI chat to let it know about the folder");
        ui.label("Navigate between messages with the left/right arrow keys");
        ui.label("Pasting with CTRL + V also advances to the next message");

        ui.label(egui::RichText::new(&self.clipboard_content_information_message).color(egui::Color32::GOLD));
    }

    fn should_allow_user_to_interact_with_settings(&self) -> bool
    {
        self.state == FolderToAiState::ReadyForUse
    }

    fn show_settings_gui(&mut self, ui: &mut egui::Ui)
    {
        egui::CollapsingHeader::new("Settings").enabled(self.should_allow_user_to_interact_with_settings()).show(ui, |ui| {
            self.settings.show_gui(ui);
        });
    }

    fn show_gui(&mut self, ui: &mut egui::Ui)
    {
        self.show_loading_progress_gui(ui);
        ui.separator();
        self.show_folder_information_gui(ui);
        ui.separator();
        self.show_messages_gui(ui);
        ui.separator();
        self.show_settings_gui(ui);
        ui.separator();
        ui.label(RichText::new("Press CTRL + SHIFT + D at any time to forcibly terminate FolderToAI").color(Color32::LIGHT_BLUE));
        ui.add(egui::Hyperlink::from_label_and_url("Feel free to take a look at the source code and/or contribute", LINK_TO_GIT_REPO));
    }

    fn update_state(&mut self)
    {
        match self.state {
            FolderToAiState::Initializing => {
                self.state = FolderToAiState::ObtainingInitialInformationAboutTheFiles;
            },
            FolderToAiState::ObtainingInitialInformationAboutTheFiles => {
                if let Ok(folder_info) = FolderInfo::new(&self.folder_path, &self.settings) {
                    self.folder_info = Some(folder_info);
                    self.state = FolderToAiState::DeterminingBinaryFiles;
                    self.total_n_files = self.folder_info.as_mut().unwrap().get_number_of_files();
                }
            },
            FolderToAiState::DeterminingBinaryFiles => {
                if self.total_n_files > 0 {
                    self.folder_info.as_mut().unwrap().determine_binarity_of_next_file(self.n_files_already_determined_whether_binary_or_not);
                    self.n_files_already_determined_whether_binary_or_not += 1;
    
                    if self.total_n_files > 0 {
                        self.progress_of_determining_binary_files = self.n_files_already_determined_whether_binary_or_not as f64 / self.total_n_files as f64;
                    }
                    else {
                        self.progress_of_determining_binary_files = 1.0;
                    }
                }
                else {
                    self.progress_of_determining_binary_files = 1.0;
                }
    
                if self.total_n_files == self.n_files_already_determined_whether_binary_or_not {
                    // files larger than 100 KiB are considered too large
                    // obs: this happens so fast that progress doesn't need to be shown
                    const MAX_FILE_SIZE_IN_BYTES: usize = 100 * 1024;
                    self.folder_info.as_mut().unwrap().determine_files_too_large(MAX_FILE_SIZE_IN_BYTES);
    
                    self.total_n_files_to_load = self.folder_info.as_mut().unwrap().get_number_of_files_whose_contents_should_be_loaded();
    
                    self.n_binary_files = self.folder_info.as_mut().unwrap().number_of_binary_files();
    
                    self.state = FolderToAiState::LoadingContents;
                }    
            },
            FolderToAiState::LoadingContents => {
                if self.total_n_files > 0 {
                    let did_load_file = self.folder_info.as_mut().unwrap().load_next_file_content_if_required(self.n_files_already_loaded_if_required_loading);
                    self.n_files_already_loaded_if_required_loading += 1;
                    if did_load_file {
                        self.n_files_loaded += 1;
                    }
    
                    if self.total_n_files_to_load > 0 {
                        self.progress_of_loading_contents = self.n_files_loaded as f64 / self.total_n_files_to_load as f64;
                    }
                    else {
                        self.progress_of_loading_contents = 1.0;
                    }
                }
                else {
                    self.progress_of_loading_contents = 1.0;
                }
    
                if self.total_n_files == self.n_files_already_loaded_if_required_loading {
                    self.n_files_that_could_not_be_loaded = self.folder_info.as_mut().unwrap().number_of_files_that_could_not_be_loaded();
                    
                    self.state = FolderToAiState::ProcessingContents;
                }
            },
            FolderToAiState::ProcessingContents => {
                if let Ok(folder_representation_messages) = core_utils::obtain_folder_representation_messages(&self.folder_info.as_mut().unwrap()) {
                    self.folder_representation_messages = folder_representation_messages;
    
                    if !self.folder_representation_messages.is_empty() {
                        clipboard_utils::set_clipboard_content(&self.folder_representation_messages[self.current_selected_message_index]);
                        self.clipboard_content_information_message = format!("Clipboard has been set to message {} of {}", self.current_selected_message_index + 1, self.folder_representation_messages.len());
                    }
    
                    self.state = FolderToAiState::ReadyForUse;
                }
                else {
                    self.state = FolderToAiState::Error;
                }
            },
            _ => {}
        }
    }
}

impl eframe::App for FolderToAiApp {
    fn on_exit(&mut self, _: Option<&eframe::glow::Context>)
    {
        self.settings.save_to_file(&setup_utils::settings_file_path());
    }

    fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_gui(ui);
        });
        
        let current_window_size = ctx.screen_rect().size();
        if current_window_size != self.previous_window_size {
            self.on_window_resize(&current_window_size);
            self.previous_window_size = current_window_size;
        }

        if self.gui_has_ever_been_updated {
            if self.state != FolderToAiState::ReadyForUse {
                let update_start_instant = Instant::now();

                loop {
                    if self.state != FolderToAiState::ReadyForUse {
                        let was_initializing = self.state == FolderToAiState::Initializing;

                        self.update_state();

                        if was_initializing {
                            break;
                        }
                    }

                    let duration_since_update_start = update_start_instant.elapsed();
                    if duration_since_update_start.as_millis() >= GUI_UPDATE_DELAY_MS {
                        break;
                    }
                }
            }
            else {
                self.deal_with_user_input();
            }
        }
        else {
            self.on_first_gui_update(ctx);
            self.gui_has_ever_been_updated = true;
        }

        if self.state == FolderToAiState::ReadyForUse {
            ctx.request_repaint_after(std::time::Duration::from_millis(GUI_UPDATE_DELAY_MS as u64));
        }
    }
}

fn on_invoked_for_folder(folder_path: String)
{
    let mut icon_data: Option<Arc<egui::viewport::IconData>> = None;
    if let Ok(icon) = image::open(setup_utils::setup_icon_path()) {
        let icon_rgba8 = icon.to_rgba8();
        let (icon_width, icon_height) = icon_rgba8.dimensions();
        icon_data = Some(Arc::new(egui::viewport::IconData {
            rgba: icon_rgba8.into_raw(),
            width: icon_width,
            height: icon_height
        }));
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([100.0, 100.0])
            .with_visible(false)
            .with_icon(icon_data.unwrap_or_else(|| Arc::new(egui::viewport::IconData::default()))),
        ..Default::default()
    };

    // due to the synchronous nature of how the application is currently structured, it is possible that the user experences unresponsiveness in the program
    // to partially mitigate this, we offer the possibility of forcibly closing the program with a hotkey
    let may_terminate_thread_flag = Arc::new(AtomicBool::new(false));
    let may_terminate_thread_flag_clone = Arc::clone(&may_terminate_thread_flag);
    let allow_to_forcibly_terminate_thread_handle = thread::spawn(move || {
        while !may_terminate_thread_flag_clone.load(Ordering::Relaxed) {
            if InputManager::new().is_control_shift_d_pressed() {
                std::process::exit(1);
            }
            thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    let folder_path_clone = folder_path.clone();
    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|_| Ok(Box::new(FolderToAiApp::new(folder_path_clone)))),
    );

    may_terminate_thread_flag.store(true, Ordering::Relaxed);
    allow_to_forcibly_terminate_thread_handle.join().unwrap();
}

fn on_manual_run()
{
    if setup_utils::is_being_executed_from_installation_location() {
        win_utils::message_box("FolderToAI is being executed manually from the installation location; that is not how it's intended to work. Right click inside a folder to use it.\n\nExecute from somewhere else to see setup options.", APP_NAME);
        return;
    }

    if setup_utils::is_installed() {
        let user_wants_to_uninstall = win_utils::yesno_message_box("Do you wish to uninstall FolderToAI?", APP_NAME);
        if user_wants_to_uninstall {
            setup_utils::uninstall();
            if setup_utils::is_completely_uninstalled() {
                win_utils::message_box("FolderToAI has been successfully uninstalled.", APP_NAME);
            }
            else {
                win_utils::message_box("The uninstallation steps were performed, however, FolderToAI could not be totally uninstalled.", APP_NAME);
            }
        }
    }
    else {
        let user_wants_to_install = win_utils::yesno_message_box("Do you wish to install FolderToAI?", APP_NAME);
        if user_wants_to_install {
            if setup_utils::setup() {
                win_utils::message_box("Setup successful.\n\nRight click inside a folder to use FolderToAI.", APP_NAME);
            }
            else {
                win_utils::message_box("Unfortunately something went wrong with the installation.", APP_NAME);
                setup_utils::uninstall();
            }
        }
    }
}


fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let folder_path: String = args[1].clone();

        on_invoked_for_folder(folder_path);
    }
    else {
        on_manual_run();
    }
}