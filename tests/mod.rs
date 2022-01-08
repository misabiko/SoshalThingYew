use serde_json::json;
use wasm_bindgen_test::*;

use soshalthing_yew::services::twitter::article::parse_text;

#[wasm_bindgen_test]
fn test_parse_plain_text() {
	let entities = json!({"urls": []});
	let extended_entities = json!({"media": []});

	let parsed = parse_text(" Plain text ".to_owned(), &entities, &extended_entities);
	assert_eq!(parsed, "Plain text".to_owned());
}