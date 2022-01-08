use serde_json::json;
use wasm_bindgen_test::*;

use soshalthing_yew::services::twitter::article::parse_text;

//wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_parse_plain_text() {
	let entities = json!({"urls": []});
	let extended_entities = json!({"media": []});

	let parsed = parse_text(" Plain text ".to_owned(), &entities, &extended_entities);
	assert_eq!(parsed, "Plain text".to_owned());
}

#[wasm_bindgen_test]
fn test_parse_media_text() {
	let entities = serde_json::from_str(r#"{
		"hashtags": [],
		"media": [
			{
				"display_url": "pic.twitter.com/rZDAWjVPA6",
				"expanded_url": "https://twitter.com/misabiko/status/1477891208819130370/photo/1",
				"ext_alt_text": null,
				"id": 1477890908666441700,
				"indices": [
					38,
					61
				],
				"media_url": "http://pbs.twimg.com/media/FIKF84PXsAETzSN.jpg",
				"media_url_https": "https://pbs.twimg.com/media/FIKF84PXsAETzSN.jpg",
				"sizes": {
					"large": {
						"h": 2048,
						"resize": "fit",
						"w": 1448
					},
					"medium": {
						"h": 1200,
						"resize": "fit",
						"w": 849
					},
					"small": {
						"h": 680,
						"resize": "fit",
						"w": 481
					},
					"thumb": {
						"h": 150,
						"resize": "crop",
						"w": 150
					}
				},
				"source_status_id": null,
				"type": "photo",
				"url": "https://t.co/rZDAWjVPA6",
				"video_info": null
			}
		],
		"symbols": [],
		"urls": [],
		"user_mentions": []
	  }"#).unwrap();
	let extended_entities = serde_json::from_str(r#"{
		"media": [
			{
				"display_url": "pic.twitter.com/rZDAWjVPA6",
				"expanded_url": "https://twitter.com/misabiko/status/1477891208819130370/photo/1",
				"ext_alt_text": null,
				"id": 1477890908666441700,
				"indices": [
					38,
					61
				],
				"media_url": "http://pbs.twimg.com/media/FIKF84PXsAETzSN.jpg",
				"media_url_https": "https://pbs.twimg.com/media/FIKF84PXsAETzSN.jpg",
				"sizes": {
					"large": {
						"h": 2048,
						"resize": "fit",
						"w": 1448
					},
					"medium": {
						"h": 1200,
						"resize": "fit",
						"w": 849
					},
					"small": {
						"h": 680,
						"resize": "fit",
						"w": 481
					},
					"thumb": {
						"h": 150,
						"resize": "crop",
						"w": 150
					}
				},
				"source_status_id": null,
				"type": "photo",
				"url": "https://t.co/rZDAWjVPA6",
				"video_info": null
			}
		]
	}"#).unwrap();

	let parsed = parse_text("drawing each one bigger than the last https://t.co/rZDAWjVPA6".to_owned(), &entities, &extended_entities);
	assert_eq!(parsed, "drawing each one bigger than the last".to_owned());
}

#[wasm_bindgen_test]
fn test_parse_url_text() {
	let entities = serde_json::from_str(r#"{
		"hashtags": [],
		"media": null,
		"symbols": [],
		"urls": [
			{
				"display_url": "twitch.tv/misabiko",
				"expanded_url": "http://twitch.tv/misabiko",
				"indices": [
					18,
					41
				],
				"url": "https://t.co/8HoM9OlDYk"
			}
		],
		"user_mentions": []
	}"#).unwrap();
	let extended_entities = serde_json::from_str("null").unwrap();

	let parsed = parse_text("trying out th17.5\nhttps://t.co/8HoM9OlDYk".to_owned(), &entities, &extended_entities);
	assert_eq!(parsed, "trying out th17.5\ntwitch.tv/misabiko".to_owned());
}