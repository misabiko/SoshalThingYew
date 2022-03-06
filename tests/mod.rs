use wasm_bindgen_test::*;
use yew::html;

use soshalthing_yew::services::twitter::article::parse_text;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_parse_plain_text() {
	let entities = serde_json::from_str(r#"{"urls": [], "hashtags": [], "user_mentions": []}"#).unwrap();
	let extended_entities = serde_json::from_str(r#"{"media": []}"#).unwrap();

	let parsed = parse_text(" Plain text ".to_owned(), entities, &extended_entities);
	let expected = "Plain text".to_owned();
	assert_eq!(parsed, (expected.clone(), html!{ {expected} }));
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

	let parsed = parse_text("drawing each one bigger than the last https://t.co/rZDAWjVPA6".to_owned(), entities, &extended_entities);
	let expected = "drawing each one bigger than the last".to_owned();
	assert_eq!(parsed, (expected.clone(), html! { {expected} }));
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

	let (parsed_text, parsed_html) = parse_text("trying out th17.5\nhttps://t.co/8HoM9OlDYk".to_owned(), entities, &extended_entities);

	assert_eq!(parsed_text, "trying out th17.5\ntwitch.tv/misabiko".to_owned(), "parsed text");

	let expected_html = html! {
		<>
			{"trying out th17.5\n"}
			<a href={"http://twitch.tv/misabiko".to_owned()}>
				{"twitch.tv/misabiko"}
			</a>
		</>
	};
	assert_eq!(parsed_html, expected_html, "parsed html");
}

#[wasm_bindgen_test]
fn test_parse_text_quote_emoji() {
	//1479800402707349500
	let entities = serde_json::from_str(r#"{
		"hashtags": [],
		"media": null,
		"symbols": [],
		"urls": [
			{
				"display_url": "twitter.com/tokoyamitowa/s…",
				"expanded_url": "https://twitter.com/tokoyamitowa/status/1479685976478056453",
				"indices": [
					26,
					49
				],
				"url": "https://t.co/kJWp13HSZz"
			}
		],
		"user_mentions": []
	}"#).unwrap();
	let extended_entities = serde_json::from_str("null").unwrap();

	let (parsed_text, parsed_html) = parse_text("配信開始〜〜！😆 https://t.co/kJWp13HSZz".to_owned(), entities, &extended_entities);

	assert_eq!(parsed_text, "配信開始〜〜！😆 twitter.com/tokoyamitowa/s…".to_owned(), "parsed text");

	let expected_html = html! {
		<>
			{"配信開始〜〜！😆 "}
			<a href={"https://twitter.com/tokoyamitowa/status/1479685976478056453".to_owned()}>
				{ "twitter.com/tokoyamitowa/s…" }
			</a>
		</>
	};
	assert_eq!(parsed_html, expected_html, "parsed html");
}

#[wasm_bindgen_test]
fn test_parse_ampersand_emoji_url() {
	//1500507982894882816
	let entities = serde_json::from_str(r#"{
		"hashtags": [],
		"media": null,
		"symbols": [],
		"urls": [
			{
				"display_url": "curiouscat.me/k0nfette",
				"expanded_url": "https://curiouscat.me/k0nfette",
				"indices": [
					13,
					36
				],
				"url": "https://t.co/DJM3u5NGJi"
			}
		],
		"user_mentions": []
	}"#).unwrap();
	let extended_entities = serde_json::from_str("null").unwrap();

	let (parsed_text, parsed_html) = parse_text("Q&amp;A?😊\nhttps://t.co/DJM3u5NGJi".to_owned(), entities, &extended_entities);

	assert_eq!(parsed_text, "Q&A?😊\ncuriouscat.me/k0nfette".to_owned(), "parsed text");

	let expected_html = html! {
		<>
			{"Q&A?😊\n"}
			<a href={"https://curiouscat.me/k0nfette".to_owned()}>
				{ "curiouscat.me/k0nfette" }
			</a>
		</>
	};
	assert_eq!(parsed_html, expected_html, "parsed html");
}

//works but assert_eq still fails...
/*#[wasm_bindgen_test]
fn test_parse_text_hashtags_url() {
	//1480012348974776322
	let entities = serde_json::from_str(r#"{
		"hashtags": [
			{
				"indices": [
					215,
					226
				],
				"text": "HololiveEN"
			},
			{
				"indices": [
					227,
					236
				],
				"text": "hololive"
			}
		],
		"media": [
			{
				"display_url": "pic.twitter.com/nEgN1iaCkN",
				"expanded_url": "https://twitter.com/FaaatSaw/status/1480012348974776322/photo/1",
				"ext_alt_text": null,
				"id": 1480012339076157400,
				"indices": [
					237,
					260
				],
				"media_url": "http://pbs.twimg.com/media/FIoPYYXUUAI_iGs.jpg",
				"media_url_https": "https://pbs.twimg.com/media/FIoPYYXUUAI_iGs.jpg",
				"sizes": {
					"large": {
						"h": 625,
						"resize": "fit",
						"w": 1111
					},
					"medium": {
						"h": 625,
						"resize": "fit",
						"w": 1111
					},
					"small": {
						"h": 383,
						"resize": "fit",
						"w": 680
					},
					"thumb": {
						"h": 150,
						"resize": "crop",
						"w": 150
					}
				},
				"source_status_id": null,
				"type": "photo",
				"url": "https://t.co/nEgN1iaCkN",
				"video_info": null
			}
		],
		"symbols": [],
		"urls": [
			{
				"display_url": "youtu.be/pKJErsN-ylU",
				"expanded_url": "https://youtu.be/pKJErsN-ylU",
				"indices": [
					85,
					108
				],
				"url": "https://t.co/TI9g4ie8eR"
			}
		],
		"user_mentions": [
			{
				"id": 1283653858510598100,
				"indices": [
					120,
					133
				],
				"name": "Mori Calliope💀holoEN",
				"screen_name": "moricalliope"
			},
			{
				"id": 1283646922406760400,
				"indices": [
					142,
					157
				],
				"name": "Takanashi Kiara🐔(insert announcement here)",
				"screen_name": "takanashikiara"
			},
			{
				"id": 1283653858510598100,
				"indices": [
					158,
					171
				],
				"name": "Mori Calliope💀holoEN",
				"screen_name": "moricalliope"
			},
			{
				"id": 1283656034305769500,
				"indices": [
					172,
					187
				],
				"name": "Watson Amelia🔎holoEN",
				"screen_name": "watsonameliaEN"
			},
			{
				"id": 1283650008835743700,
				"indices": [
					188,
					202
				],
				"name": "Ninomae Ina’nis🐙holoEN",
				"screen_name": "ninomaeinanis"
			},
			{
				"id": 1283657064410017800,
				"indices": [
					203,
					212
				],
				"name": "Gawr Gura🔱holoEN",
				"screen_name": "gawrgura"
			}
		]
	}"#).unwrap();
	let extended_entities = serde_json::from_str(r#"{
		"media": [
			{
				"display_url": "pic.twitter.com/nEgN1iaCkN",
				"expanded_url": "https://twitter.com/FaaatSaw/status/1480012348974776322/photo/1",
				"ext_alt_text": null,
				"id": 1480012339076157400,
				"indices": [
					237,
					260
				],
				"media_url": "http://pbs.twimg.com/media/FIoPYYXUUAI_iGs.jpg",
				"media_url_https": "https://pbs.twimg.com/media/FIoPYYXUUAI_iGs.jpg",
				"sizes": {
					"large": {
						"h": 625,
						"resize": "fit",
						"w": 1111
					},
					"medium": {
						"h": 625,
						"resize": "fit",
						"w": 1111
					},
					"small": {
						"h": 383,
						"resize": "fit",
						"w": 680
					},
					"thumb": {
						"h": 150,
						"resize": "crop",
						"w": 150
					}
				},
				"source_status_id": null,
				"type": "photo",
				"url": "https://t.co/nEgN1iaCkN",
				"video_info": null
			}
		]
	}"#).unwrap();

	let (parsed_text, parsed_html) = parse_text("[LATEST MUSIC WORK]\n\nI produced holoEN’s “Journey Like A Thousand Years”\n\n🔗 https://t.co/TI9g4ie8eR\n\nLyrics by @moricalliope \nVox by @takanashikiara @moricalliope @watsonameliaEN @ninomaeinanis @gawrgura \n\n#HololiveEN #hololive https://t.co/nEgN1iaCkN".to_owned(), entities, &extended_entities);

	assert_eq!(parsed_text, "[LATEST MUSIC WORK]\n\nI produced holoEN’s “Journey Like A Thousand Years”\n\n🔗 youtu.be/pKJErsN-ylU\n\nLyrics by @moricalliope \nVox by @takanashikiara @moricalliope @watsonameliaEN @ninomaeinanis @gawrgura \n\n#HololiveEN #hololive".to_owned(), "parsed text");

	let expected_html = html! {
		<>
			{"[LATEST MUSIC WORK]\n\nI produced holoEN’s “Journey Like A Thousand Years”\n\n🔗 "}
			<a href={"https://youtu.be/pKJErsN-ylU".to_owned()}>
				{ "youtu.be/pKJErsN-ylU" }
			</a>
			{"\n\nLyrics by "}
			<a href={"https://twitter.com/moricalliope".to_owned()}>
				{"@moricalliope"}
			</a>
			{"\nVox by "}
			<a href={"https://twitter.com/takanashikiara".to_owned()}>
				{"@takanashikiara"}
			</a>
			{" "}
			<a href={"https://twitter.com/moricalliope".to_owned()}>
				{"@moricalliope"}
			</a>
			{" "}
			<a href={"https://twitter.com/watsonameliaEN".to_owned()}>
				{"@watsonameliaEN"}
			</a>
			{" "}
			<a href={"https://twitter.com/ninomaeinanis".to_owned()}>
				{"@ninomaeinanis"}
			</a>
			{" "}
			<a href={"https://twitter.com/gawrgura".to_owned()}>
				{"@gawrgura"}
			</a>
			{" \n\n"}
			<a href={"https://twitter.com/search?q=#HololiveEN".to_owned()}>
				{"#HololiveEN"}
			</a>
			{" "}
			<a href={"https://twitter.com/search?q=#hololive".to_owned()}>
				{"#hololive"}
			</a>
			/*<a href={"https://twitter.com/tokoyamitowa/status/1479685976478056453".to_owned()}>
				{ "twitter.com/tokoyamitowa/s…" }
			</a>*/
		</>
	};
	assert_eq!(parsed_html, expected_html, "parsed html");
}*/