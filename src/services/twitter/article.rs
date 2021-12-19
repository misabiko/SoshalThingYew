use std::rc::{Rc, Weak};
use std::cell::{RefCell, Ref};
use js_sys::Date;
use wasm_bindgen::JsValue;
use std::collections::HashSet;

use crate::articles::{ArticleData, ArticleMedia, ArticleRefType};

#[derive(Clone, PartialEq)]
pub struct TwitterUser {
	pub username: String,
	pub name: String,
	pub avatar_url: String,
}

pub struct TweetArticleData {
	pub id: u64,
	pub text: Option<String>,
	pub author: TwitterUser,
	pub creation_time: Date,
	pub liked: bool,
	pub retweeted: bool,
	pub like_count: i64,	//TODO Try casting i64 to i32
	pub retweet_count: i64,
	pub media: Vec<ArticleMedia>,
	pub raw_json: serde_json::Value,
	pub referenced_article: ArticleRefType,
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
	fn creation_time(&self) -> Date {
		self.creation_time.clone()
	}
	fn text(&self) -> String {
		self.text.clone().unwrap_or("".to_owned())
	}
	fn author_username(&self) -> String {
		self.author.username.clone()
	}
	fn author_name(&self) -> String {
		self.author.name.clone()
	}
	fn author_avatar_url(&self) -> String {
		self.author.avatar_url.clone()
	}
	fn author_url(&self) -> String {
		format!("https://twitter.com/{}", &self.author.username)
	}
	fn like_count(&self) -> i64 {
		self.like_count.clone()
	}
	fn repost_count(&self) -> i64 {
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
		self.referenced_article.clone()
	}
	fn url(&self) -> String {
		format!("https://twitter.com/{}/status/{}", &self.author_username(), &self.id())
	}
	fn update(&mut self, new: &Ref<dyn ArticleData>) {
		self.liked = new.liked();
		self.retweeted = new.reposted();
		self.like_count = new.like_count();
		self.retweet_count = new.repost_count();
		self.raw_json = new.json();
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
}

pub type StrongArticleRefType = ArticleRefType<Rc<RefCell<TweetArticleData>>>;

impl TweetArticleData {
	pub fn from(json: &serde_json::Value, marked_as_read: &HashSet<u64>) -> (Rc<RefCell<Self>>, StrongArticleRefType) {
		let id = json["id"].as_u64().unwrap();

		let referenced_article: StrongArticleRefType = {
			let referenced = &json["retweeted_status"];
			let quoted = &json["quoted_status"];

			if !referenced.is_null() {
				let parsed = TweetArticleData::from(&referenced.clone(), &marked_as_read);
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
				let parsed = TweetArticleData::from(&quoted.clone(), &marked_as_read);
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

		let medias_opt = json["extended_entities"]
			.get("media")
			.and_then(|media| media.as_array());

		let data = Rc::new(RefCell::new(TweetArticleData {
			id,
			creation_time: json["created_at"].as_str().map(|datetime_str|Date::new(&JsValue::from_str(datetime_str))).unwrap(),
			text: match json["full_text"].as_str() {
				Some(text) => Some(text),
				None => json["text"].as_str()
			}.map(String::from),
			author: TwitterUser {
				username: json["user"]["screen_name"].as_str().unwrap().to_owned(),
				name: json["user"]["name"].as_str().unwrap().to_owned(),
				avatar_url: json["user"]["profile_image_url_https"].as_str().unwrap().to_owned(),
			},
			liked: json["favorited"].as_bool().unwrap_or_default(),
			retweeted: json["retweeted"].as_bool().unwrap_or_default(),
			like_count: json["favorite_count"].as_i64().unwrap(),
			retweet_count: json["retweet_count"].as_i64().unwrap(),
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
														.map(|(w, h)| h as f32 / w as f32)
												))
											.map(|(url, ratio)| ArticleMedia::Image(url.to_owned(), ratio)),
									"animated_gif" => m.get("video_info")
										.and_then(|v| get_mp4(v))
										.map(|(url, ratio)| ArticleMedia::Gif(url.to_owned(), ratio)),
									"video" => m.get("video_info")
										.and_then(|v| get_mp4(v))
										.map(|(url, ratio)| ArticleMedia::Video(url.to_owned(), ratio)),
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
				StrongArticleRefType::Repost(a) => ArticleRefType::Repost(Rc::downgrade(a) as Weak<RefCell<dyn ArticleData>>),
				StrongArticleRefType::Quote(a) => ArticleRefType::Quote(Rc::downgrade(a) as Weak<RefCell<dyn ArticleData>>),
				StrongArticleRefType::QuoteRepost(quote, quoted) => ArticleRefType::QuoteRepost(
					Rc::downgrade(quote) as Weak<RefCell<dyn ArticleData>>,
					Rc::downgrade(quoted) as Weak<RefCell<dyn ArticleData>>
				),
			},
			marked_as_read: marked_as_read.contains(&id),
			hidden: false,
		}));
		(data, referenced_article)
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

fn get_mp4(video_info: &serde_json::Value) -> Option<(&str, f32)> {
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
				.map(|(w, h)| h as f32 / w as f32)
		)
}