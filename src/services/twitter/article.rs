use std::rc::{Rc, Weak};
use std::cell::{RefCell, Ref};
use js_sys::Date;
use wasm_bindgen::JsValue;
use std::num::{NonZeroU32, NonZeroU16};
use serde::Deserialize;
use yew::prelude::*;

use crate::articles::{ArticleData, ArticleMedia, MediaType, MediaQueueInfo, ArticleRefType, ValidRatio};
use crate::services::storages::ServiceStorage;

#[derive(Deserialize)]
pub struct Entities {
	hashtags: Vec<TweetHashtag>,
	// media: Vec<>
	// symbols: Vec<>
	urls: Vec<TweetUrl>,
	user_mentions: Vec<TweetMention>,
}

#[derive(Deserialize)]
pub struct TweetUrl {
	display_url: String,
	expanded_url: String,
	indices: (usize, usize),
	url: String,
}

#[derive(Deserialize)]
pub struct TweetHashtag {
	indices: (usize, usize),
	text: String,
}

#[derive(Deserialize)]
pub struct TweetMention {
	indices: (usize, usize),
	// id: u64,
	// name: String,
	screen_name: String,
}

#[derive(Deserialize)]
pub struct ExtendedEntities {
	media: Vec<TweetMedia>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TweetMedia {
	Photo {
		// display_url: String
		// expanded_url: String
		// ext_alt_text: null
		// id: u32
		// indices: (u16, u16)
		// media_url: String
		media_url_https: String,
		sizes: TweetMediaSizes,
		// source_status_id: null
		// type: "photo"
		url: String,
		// video_info: null
	},
	AnimatedGif {
		// display_url: String
		// expanded_url: String
		// ext_alt_text: null
		// id: u32
		// indices: (u16, u16)
		// media_url: String
		// media_url_https: String,
		// sizes: TweetMediaSizes,
		// source_status_id: null
		// type: "photo"
		url: String,
		video_info: VideoInfo,
	},
	Video {
		// display_url: String
		// expanded_url: String
		// ext_alt_text: null
		// id: u32
		// indices: (u16, u16)
		// media_url: String
		// media_url_https: String,
		// sizes: TweetMediaSizes,
		// source_status_id: null
		// type: "photo"
		url: String,
		video_info: VideoInfo,
	},
}

#[derive(Deserialize)]
pub struct TweetMediaSizes {
	large: TweetMediaSize,
	// medium: TweetMediaSize,
	// small: TweetMediaSize,
	// thumb: TweetMediaSize,
}

#[derive(Deserialize)]
pub struct TweetMediaSize {
	h: u32,
	//resize: enum
	w: u32,
}

#[derive(Deserialize)]
pub struct VideoInfo {
	aspect_ratio: (u16, u16),
	// duration_millis: u32,
	variants: Vec<VideoVariant>,
}

#[derive(Deserialize)]
pub struct VideoVariant {
	// bitrate: u32,
	content_type: String,
	// enum
	url: String,
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub struct TwitterUser {
	pub username: String,
	pub name: String,
	pub avatar_url: String,
}

pub type StrongArticleRefType = ArticleRefType<Rc<RefCell<TweetArticleData>>>;

#[derive(Clone, Debug)]
pub struct TweetArticleData {
	pub id: u64,
	pub text: String,
	pub author: TwitterUser,
	pub creation_time: Date,
	pub liked: bool,
	pub retweeted: bool,
	pub like_count: u32,
	pub retweet_count: u32,
	pub media: Vec<ArticleMedia>,
	pub raw_json: serde_json::Value,
	pub referenced_article: ArticleRefType<Weak<RefCell<TweetArticleData>>>,
	pub marked_as_read: bool,
	pub hidden: bool,
	pub text_html: Html,
}

impl ArticleData for TweetArticleData {
	fn service(&self) -> &'static str {
		"Twitter"
	}
	fn id(&self) -> String {
		self.id.to_string()
	}
	fn sortable_id(&self) -> usize {
		self.id as usize
	}
	fn creation_time(&self) -> Date {
		self.creation_time.clone()
	}
	fn text(&self) -> String {
		self.text.clone()
	}
	fn author_name(&self) -> String {
		self.author.name.clone()
	}
	fn author_username(&self) -> String {
		self.author.username.clone()
	}
	fn author_avatar_url(&self) -> String {
		self.author.avatar_url.clone()
	}
	fn author_url(&self) -> String {
		format!("https://twitter.com/{}", &self.author.username)
	}
	fn like_count(&self) -> u32 {
		self.like_count
	}
	fn repost_count(&self) -> u32 {
		self.retweet_count
	}
	fn liked(&self) -> bool {
		self.liked
	}
	fn reposted(&self) -> bool {
		self.retweeted
	}
	fn media(&self) -> Vec<ArticleMedia> {
		self.media.clone()
	}
	fn json(&self) -> serde_json::Value { self.raw_json.clone() }
	fn referenced_article(&self) -> ArticleRefType {
		match &self.referenced_article {
			ArticleRefType::NoRef => ArticleRefType::NoRef,
			ArticleRefType::Repost(a) => ArticleRefType::Repost(a.clone() as Weak<RefCell<dyn ArticleData>>),
			ArticleRefType::Quote(a) => ArticleRefType::Quote(a.clone() as Weak<RefCell<dyn ArticleData>>),
			ArticleRefType::QuoteRepost(a, q) => ArticleRefType::QuoteRepost(
				a.clone() as Weak<RefCell<dyn ArticleData>>,
				q.clone() as Weak<RefCell<dyn ArticleData>>,
			),
		}
	}
	fn url(&self) -> String {
		format!("https://twitter.com/{}/status/{}", &self.author_username(), &self.id())
	}
	fn marked_as_read(&self) -> bool {
		self.marked_as_read
	}
	fn set_marked_as_read(&mut self, value: bool) {
		self.marked_as_read = value;
	}
	fn hidden(&self) -> bool {
		self.hidden
	}
	fn set_hidden(&mut self, value: bool) {
		self.hidden = value;
	}
	fn clone_data(&self) -> Box<dyn ArticleData> {
		Box::new(self.clone())
	}
	fn media_loaded(&mut self, _index: usize) {
		log::warn!("Twitter doesn't do lazy loading.");
	}
	fn view_text(&self) -> Html {
		self.text_html.clone()
	}
}

impl TweetArticleData {
	pub fn from(json: &serde_json::Value, storage: &ServiceStorage) -> (Rc<RefCell<Self>>, StrongArticleRefType) {
		let id = json["id"].as_u64().unwrap();

		let referenced_article: StrongArticleRefType = {
			let referenced = &json["retweeted_status"];
			let quoted = &json["quoted_status"];

			if !referenced.is_null() {
				let parsed = TweetArticleData::from(&referenced.clone(), storage);
				match parsed.1 {
					ArticleRefType::NoRef => StrongArticleRefType::Repost(parsed.0),
					ArticleRefType::Quote(parsed_ref) => StrongArticleRefType::QuoteRepost(parsed.0, parsed_ref),
					ArticleRefType::Repost(parsed_ref) => {
						log::warn!("Retweet({}) of a retweet({})?", &id, &parsed_ref.borrow().id());
						StrongArticleRefType::Repost(parsed.0)
					}
					ArticleRefType::QuoteRepost(parsed_ref, parsed_quoted) => {
						log::warn!("Retweet({}) of a retweet({}) of a quote({})??", &id, &parsed_ref.borrow().id(), &parsed_quoted.borrow().id());
						StrongArticleRefType::Repost(parsed.0)
					}
				}
			}else if !quoted.is_null() {
				let parsed = TweetArticleData::from(&quoted.clone(), storage);
				match parsed.1 {
					ArticleRefType::NoRef => StrongArticleRefType::Quote(parsed.0),
					ArticleRefType::Quote(parsed_ref) => {
						log::warn!("Quote({}) of a quote({})?", &id, &parsed_ref.borrow().id());
						StrongArticleRefType::Quote(parsed.0)
					}
					ArticleRefType::Repost(parsed_ref) => {
						log::warn!("Retweet({}) of a retweet({})?", &id, &parsed_ref.borrow().id());
						StrongArticleRefType::Quote(parsed.0)
					}
					ArticleRefType::QuoteRepost(parsed_ref, parsed_quoted) => {
						log::warn!("Retweet({}) of a retweet({}) of a quote({})??", &id, &parsed_ref.borrow().id(), &parsed_quoted.borrow().id());
						StrongArticleRefType::Quote(parsed.0)
					}
				}
			}else {
				StrongArticleRefType::NoRef
			}
		};

		let extended_entities: Option<ExtendedEntities> = serde_json::from_value(json["extended_entities"].clone()).unwrap();

		let text = match json["full_text"].as_str().or(json["text"].as_str()) {
			Some(text) => text,
			None => "",
		}.to_owned();

		let (text, text_html) = parse_text(text, serde_json::from_value(json["entities"].clone()).unwrap(), &extended_entities);

		let data = Rc::new(RefCell::new(TweetArticleData {
			id,
			creation_time: json["created_at"].as_str().map(|datetime_str|Date::new(&JsValue::from_str(datetime_str))).unwrap(),
			text,
			author: TwitterUser {
				username: json["user"]["screen_name"].as_str().unwrap().to_owned(),
				name: json["user"]["name"].as_str().unwrap().to_owned(),
				avatar_url: json["user"]["profile_image_url_https"].as_str().unwrap().to_owned(),
			},
			liked: json["favorited"].as_bool().unwrap_or_default(),
			retweeted: json["retweeted"].as_bool().unwrap_or_default(),
			like_count: json["favorite_count"].as_u64().unwrap() as u32,
			retweet_count: json["retweet_count"].as_u64().unwrap() as u32,
			media: parse_media(extended_entities.map(|e| e.media)),
			raw_json: json.clone(),
			referenced_article: match &referenced_article {
				StrongArticleRefType::NoRef => ArticleRefType::NoRef,
				StrongArticleRefType::Repost(a) => ArticleRefType::Repost(Rc::downgrade(a)),
				StrongArticleRefType::Quote(a) => ArticleRefType::Quote(Rc::downgrade(a)),
				StrongArticleRefType::QuoteRepost(quote, quoted) => ArticleRefType::QuoteRepost(
					Rc::downgrade(quote),
					Rc::downgrade(quoted)
				),
			},
			marked_as_read: storage.session.articles_marked_as_read.contains(&id.to_string()),
			hidden: storage.local.hidden_articles.contains(&id.to_string()),
			text_html,
		}));
		(data, referenced_article)
	}

	pub fn update(&mut self, new: &Ref<TweetArticleData>) {
		self.liked = new.liked;
		self.retweeted = new.retweeted;
		self.like_count = new.like_count;
		self.retweet_count = new.retweet_count;
		self.raw_json = new.raw_json.clone();
	}
}

fn get_mp4(video_info: &VideoInfo, media_type: MediaType) -> ArticleMedia {
	ArticleMedia {
		media_type,
		src: video_info.variants
			.iter().find(|v| v.content_type == "video/mp4").expect("finding mp4 video")
			.url.clone(),
		ratio: ValidRatio::new_u16(
			NonZeroU16::new(video_info.aspect_ratio.0).expect("non-zero width"),
			NonZeroU16::new(video_info.aspect_ratio.1).expect("non-zero height"),
		),
		queue_load_info: MediaQueueInfo::DirectLoad,
	}
}

fn parse_media(media: Option<Vec<TweetMedia>>) -> Vec<ArticleMedia> {
	match media {
		Some(medias) => {
			medias.iter()
				.map(|m| match m {
					TweetMedia::Photo { media_url_https, sizes, .. } => ArticleMedia {
						media_type: MediaType::Image,
						src: media_url_https.clone(),
						ratio: ValidRatio::new_u32(
							NonZeroU32::new(sizes.large.w).expect("non-zero width"),
							NonZeroU32::new(sizes.large.h).expect("non-zero height"),
						),
						queue_load_info: MediaQueueInfo::DirectLoad,
					},
					TweetMedia::AnimatedGif { video_info, .. } => get_mp4(video_info, MediaType::VideoGif),
					TweetMedia::Video { video_info, .. } => get_mp4(video_info, MediaType::Video),
				})
				.collect()
		}
		None => Vec::new()
	}
}

/*enum TextToken {
	Text(String),
	Link(String),
}*/

pub fn parse_text(original: String, entities: Entities, extended_entities: &Option<ExtendedEntities>) -> (String, Html) {
	let mut trimmed_text = original.clone();
	let medias_opt: Option<Vec<&String>> = extended_entities.as_ref().map(|e|
			e.media.iter().map(|m| match m {
				TweetMedia::Photo { url, .. } |
				TweetMedia::AnimatedGif { url, .. } |
				TweetMedia::Video { url, .. }
				=> url
			}).collect()
		);

	if let Some(medias) = medias_opt {
		for media in medias {
			trimmed_text = trimmed_text.replace(media, "");
		}
	}

	let mut final_text = trimmed_text.clone();

	let mut html_parts = Vec::new();
	for TweetUrl { display_url, expanded_url, indices, url } in entities.urls {
		final_text = final_text.replace(url.as_str(), display_url.as_str());
		html_parts.push((indices, html! { <a href={expanded_url.clone()}>{display_url.as_str()}</a> }))
	}
	for TweetHashtag { indices, text } in entities.hashtags {
		html_parts.push((indices, html! {
			<a href={format!("https://twitter.com/search?q=#{}", text)}>
				{format!("#{}", text)}
			</a>
		}))
	}
	for TweetMention { indices, /*id: _, name: _, */screen_name } in entities.user_mentions {
		html_parts.push((indices, html! {
			<a href={format!("https://twitter.com/{}", screen_name)}>
				{format!("@{}", screen_name)}
			</a>
		}))
	}

	final_text = final_text.trim().to_owned();

	let html = if html_parts.is_empty() {
		html! { { final_text.clone() } }
	}else {
		html_parts.sort_by(|((a, _), _), ((b, _), _)| a.cmp(b));

		let mut i = 0;
		let len = trimmed_text.len();
		let mut new_html_parts = Vec::new();
		let last_index = html_parts.iter().last().unwrap().0.1;
		for ((first, last), html) in html_parts {
			if i < first {
				new_html_parts.push(html! { {original.as_str()[i..first].to_owned()} });
			}

			new_html_parts.push(html.clone());
			i = last;
		}

		if i < len - 1 {
			new_html_parts.push(html! { {trimmed_text.as_str()[last_index..].to_owned()} });
		}

		html! { { for new_html_parts.into_iter() } }
	};

	(final_text, html)
}