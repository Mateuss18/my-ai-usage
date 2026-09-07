#[tauri::command]
fn greet() -> String {
    "Hello from Rust!".to_owned()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::greet;

    #[test]
    fn greet_response_is_deterministic() {
        assert_eq!(greet(), "Hello from Rust!");
    }
}
