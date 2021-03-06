use std::fs;
use serde_json::{json, Value};

fn main() -> std::io::Result<()> {
    let mut json = json!({});
    let paths = fs::read_dir("dist/.stage/")?;

    for path in paths {
        let path = path.ok()
            .map(|p| p.path());
        let file_name = path.as_ref().and_then(|p| p.file_name()).and_then(|s| s.to_str());
        let extension = path.as_ref().and_then(|p| p.extension()).and_then(|s| s.to_str());
        if let Some((file_name, extension)) = file_name.zip(extension) {
            let prefix = match extension {
                "css" => "partial-index-",
                _ => "index-",
            };

            if file_name.contains(prefix) {
                json[extension] = Value::String(file_name.to_string());
            }
        }
    }

    //TODO Have trunk set RUST_LOG env
    //env_logger::init();
    //log::info!("generated files {}", &json.to_string());
    println!("generated files {}", &json.to_string());

    fs::write("dist/.stage/generated_files.json", json.to_string())
}