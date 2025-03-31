use clipboard::{ClipboardContext, ClipboardProvider};

pub fn set_clipboard_content(content: &str) -> bool
{
    if let Ok(mut ctx) = <ClipboardContext>::new() {
        if let Ok(_) = ctx.set_contents(content.to_string()) {
            return true;
        }
    }
    false
}