use std::rc::{Rc, Weak};
use std::cell::{RefCell, Ref};
use js_sys::Date;
use wasm_bindgen::JsValue;
use std::num::NonZeroU64;

use crate::articles::{ArticleData, ArticleMedia, MediaType, MediaQueueInfo, ArticleRefType, ValidRatio};
use crate::services::storages::SessionStorageService;

#[derive(Clone, PartialEq, Debug, serde::Deserialize)]
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
}

impl ArticleData for TweetArticleData {
	fn service(&self) -> &'static str {
		"Twitter"
	}
	fn id(&self) -> String {
		self.id.clone().to_string()
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
		self.like_count.clone()
	}
	fn repost_count(&self) -> u32 {
		self.retweet_count.clone()
	}
	fn liked(&self) -> bool {
		self.liked.clone()
	}
	fn reposted(&self) -> bool {
		self.retweeted.clone()
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
		self.marked_as_read.clone()
	}
	fn set_marked_as_read(&mut self, value: bool) {
		self.marked_as_read = value;
	}
	fn hidden(&self) -> bool {
		self.hidden.clone()
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
}

impl TweetArticleData {
	pub fn from(json: &serde_json::Value, storage: &SessionStorageService) -> (Rc<RefCell<Self>>, StrongArticleRefType) {
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

		let extended_entities = &json["extended_entities"];
		let medias_opt = extended_entities
			.get("media")
			.and_then(|media| media.as_array());

		let mut text = match json["full_text"].as_str().or(json["text"].as_str()) {
			Some(text) => text,
			None => "",
		}.to_owned();

		text = parse_text(text, &json["entities"], &extended_entities);

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
			media: match medias_opt {
				Some(medias) => {
					medias.iter()
						.map(|m| {
							m.get("type")
								.and_then(|t| t.as_str())
								.and_then(|t| match t {
									"photo" => m.get("media_url_https")
											.and_then(|url| url.as_str())
											.zip(m.get("sizes")
												.and_then(|s| s.get("large"))
												.and_then(|s|
													s.get("w")
														.and_then(|w| w.as_u64())
														.zip(s.get("h")
															.and_then(|h| h.as_u64())
														)
														.map(|(w, h)| ValidRatio::new_u64(
															NonZeroU64::new(w).expect("non-zero width"),
															NonZeroU64::new(h).expect("non-zero height"),
														))
												))
											.map(|(url, ratio)| ArticleMedia {
												media_type: MediaType::Image,
												src: url.to_owned(),
												ratio,
												queue_load_info: MediaQueueInfo::DirectLoad,
											}),
									"animated_gif" => m.get("video_info")
										.and_then(|v| get_mp4(v))
										.map(|(url, ratio)| ArticleMedia {
											media_type: MediaType::VideoGif,
											src: url.to_owned(),
											ratio,
											queue_load_info: MediaQueueInfo::DirectLoad,
										}),
									"video" => m.get("video_info")
										.and_then(|v| get_mp4(v))
										.map(|(url, ratio)| ArticleMedia {
											media_type: MediaType::Video,
											src: url.to_owned(),
											ratio,
											queue_load_info: MediaQueueInfo::DirectLoad,
										}),
									other_type => {
										log::warn!("Unexpected media type \"{}\"", &other_type);
										None
									}
								})
						})
						.filter_map(std::convert::identity)
						.collect()
				},
				None => Vec::new()
			},
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
			marked_as_read: storage.articles_marked_as_read.contains(&id.to_string()),
			hidden: false,
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

fn first_mp4(variants: &Vec<serde_json::Value>) -> Option<&serde_json::Value> {
	variants.iter().find(|v|
		v.get("content_type")
			.and_then(|t| t.as_str())
			.map(|t| t == "video/mp4")
			.unwrap_or(false)
	)
}

fn get_mp4(video_info: &serde_json::Value) -> Option<(&str, ValidRatio)> {
	video_info.get("variants")
		.and_then(|v| v.as_array())
		.and_then(|v| first_mp4(v))
		.and_then(|v| v.get("url"))
		.and_then(|url| url.as_str())
		.zip(
			video_info.get("aspect_ratio")
				.and_then(|r| r.as_array())
				.and_then(|r| r.get(0)
					.and_then(|w| w.as_u64())
					.zip(r.get(1).and_then(|w| w.as_u64())))
				.map(|(w, h)| ValidRatio::new_u64(
					NonZeroU64::new(w).expect("non-zero width"),
					NonZeroU64::new(h).expect("non-zero height"),
				))
		)
}

pub fn parse_text(mut text: String, entities: &serde_json::Value, extended_entities: &serde_json::Value) -> String {
	let medias_opt: Option<Vec<&str>> = extended_entities
		.get("media")
		.and_then(|media| media.as_array())
		.map(|medias| medias.iter().filter_map(|m| {
			m.get("type")
				.and_then(|t| t.as_str())
				.and_then(|t| if let "photo" | "video" | "animated_gif" = t {
					m.get("url")
				}else {
					None
				})
				.and_then(|u| u.as_str())
			}).collect()
		);

	let urls_opt: Option<Vec<(&str, &str)>> = entities
		.get("urls")
		.and_then(|url| url.as_array())
		.map(|urls|
			urls.iter()
				.filter_map(|url| url["url"].as_str().zip(url["display_url"].as_str())
			).collect()
		);

	if let Some(medias) = medias_opt {
		for media in medias {
			text = text.replace(media, "");
		}
	}

	if let Some(urls) = urls_opt {
		for (compressed, display) in urls {
			text = text.replace(compressed, display);
		}
	}

	text
}