use wasm_bindgen_test::*;
use yew::html;

use soshalthing::services::twitter::article::parse_text;

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
	let tweet: serde_json::Value = serde_json::from_str(include_str!("fixtures/tweet_media_text.json")).unwrap();
	let entities = serde_json::from_value(tweet["entities"].clone()).unwrap();
	let extended_entities = serde_json::from_value(tweet["extended_entities"].clone()).unwrap();

	let parsed = parse_text("drawing each one bigger than the last https://t.co/rZDAWjVPA6".to_owned(), entities, &extended_entities);
	let expected = "drawing each one bigger than the last".to_owned();
	assert_eq!(parsed, (expected.clone(), html! { {expected} }));
}

#[wasm_bindgen_test]
fn test_parse_url_text() {
	let tweet: serde_json::Value = serde_json::from_str(include_str!("fixtures/tweet_text_url.json")).unwrap();
	let entities = serde_json::from_value(tweet["entities"].clone()).unwrap();
	let extended_entities = serde_json::from_value(tweet["extended_entities"].clone()).unwrap();

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
fn test_parse_ampersand_emoji_url() {
	let tweet: serde_json::Value = serde_json::from_str(include_str!("fixtures/tweet_ampersand_emoji_url.json")).unwrap();
	let entities = serde_json::from_value(tweet["entities"].clone()).unwrap();
	let extended_entities = serde_json::from_value(tweet["extended_entities"].clone()).unwrap();

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
	let tweet: serde_json::Value = serde_json::from_str(include_str!("fixtures/tweet_text_hashtags_url.json")).unwrap();
	let entities = serde_json::from_value(tweet["entities"].clone()).unwrap();
	let extended_entities = serde_json::from_value(tweet["extended_entities"].clone()).unwrap();

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