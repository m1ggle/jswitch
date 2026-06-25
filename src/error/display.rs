use super::JswitchError;

pub fn format_error(error: &JswitchError) -> String {
    error.to_string()
}
