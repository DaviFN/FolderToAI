use crate::folder_info::FolderInfo;

use size::Size;
use unicode_segmentation::UnicodeSegmentation;

pub fn obtain_folder_representation_messages(folder_info: &FolderInfo) -> Result<Vec<String>,()> {
    // this function can be made faster by building the parts directly from the file contents instead of concatenating all of them first

    const MAXIMUM_AMOUNT_OF_CHARACTERS_ALLOWED_IN_LLM_MODEL: usize = 4096;
    const APPROXIMATE_MAXIMUM_AMOUNT_OF_CHARACTERS_ALLOWED_IN_LLM_MODEL: usize = MAXIMUM_AMOUNT_OF_CHARACTERS_ALLOWED_IN_LLM_MODEL - 16; // assuming the LLM works with 4096 characters; leave 16 characters for starting message

    let mut concatenated_file_contents = String::new();

    let folder_contains_no_relevant_files = !folder_info.contains_at_least_one_file_that_should_not_be_ignored();
    if folder_contains_no_relevant_files {
        let empty_folder_message = format!("[FolderToAI]\n\nMessage 1/1\n\nThis message will provide you relevant information about the files within the folder {}.\n\nThe folder contains no relevant files.", folder_info.folder_path);

        let mut folder_representation_messages: Vec<String> = vec!();
        folder_representation_messages.push(empty_folder_message);
        return Ok(folder_representation_messages);
    }

    for file_info in &folder_info.file_infos {
        if file_info.should_be_ignored {
            continue;
        }

        concatenated_file_contents += &format!("File: {}\nSize: {}\n--- BEGINNING OF CONTENT ---\n", file_info.filepath, Size::from_bytes(file_info.size_in_bytes));
        if let Some(file_content) = &file_info.file_content {
            concatenated_file_contents += &file_content;
        }
        else {
            if file_info.is_binary {
                concatenated_file_contents += "[Binary file]";
            }
            else if file_info.file_too_large {
                concatenated_file_contents += "[This file is too large to be loaded]";
            }
            else if file_info.content_should_be_loaded() {
                concatenated_file_contents += "[File content could not be loaded]";
            }
            else {
                concatenated_file_contents += "[Error]";
            }
        }
        concatenated_file_contents += "\n--- END OF CONTENT ---\n";
    }

    let mut file_contents_parts: Vec<String> = vec!();
    let chars: Vec<&str> = concatenated_file_contents.as_str().graphemes(true).collect();
    let n_parts = (chars.len() + APPROXIMATE_MAXIMUM_AMOUNT_OF_CHARACTERS_ALLOWED_IN_LLM_MODEL - 1) / APPROXIMATE_MAXIMUM_AMOUNT_OF_CHARACTERS_ALLOWED_IN_LLM_MODEL;
    let total_n_messages = n_parts + 1;

    let mut current_message_index: usize = 2;

    let mut current_pos = 0;
    while current_pos < chars.len() {
        let end = std::cmp::min(current_pos + APPROXIMATE_MAXIMUM_AMOUNT_OF_CHARACTERS_ALLOWED_IN_LLM_MODEL, chars.len());
        let chunk: String = chars[current_pos..end].concat();
        
        let starting_string = format!("Message {}/{}:\n", current_message_index, total_n_messages);
        file_contents_parts.push(starting_string + &chunk);
        current_pos = end;
        current_message_index += 1;
    }

    let mut prologue = String::from("[FolderToAI]");
    prologue += &format!("\n\nMessage 1/{}:\n\nThis and the message(s) that follow will provide you relevant information about the files within the folder \"{}\", which occupies {}. There are {} messages in total.", total_n_messages, folder_info.folder_path, Size::from_bytes(folder_info.size_in_bytes).to_string(), total_n_messages);
    prologue += "\n\nAt the beginning of each message, its index will be stated, along with the total number of messages. Each file's content will be be between lines that read \"--- BEGINNING OF CONTENT ---\" and \"--- END OF CONTENT ---\". Note that these delimiters may be split in between messages but they will all eventually be there once all the parts get sent.";
    prologue += &format!("\n\nThe messages will contain at most {} characters, including line breaks. Please acknowledge that you get all the messages correctly and in sequence, given the indices provided at the beginning of each message. Warn me about any gaps (missing messages) and make sure you receive all {} of them in order.", MAXIMUM_AMOUNT_OF_CHARACTERS_ALLOWED_IN_LLM_MODEL, total_n_messages);

    let mut folder_representation_messages: Vec<String> = vec!();
    folder_representation_messages.push(prologue);
    for file_contents_part in file_contents_parts {
        folder_representation_messages.push(file_contents_part);
    }

    return Ok(folder_representation_messages);
}