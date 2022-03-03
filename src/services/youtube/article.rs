use std::cell::Ref;
use std::num::NonZeroU16;
use js_sys::Date;
use wasm_bindgen::JsValue;
use serde::Deserialize;
use derivative::Derivative;
use serde_json::Value;

use super::SERVICE_INFO;
use crate::articles::{ArticleData, ArticleMedia, MediaQueueInfo, MediaType, ValidRatio};
use crate::services::storages::ServiceStorage;

#[derive(Clone, Debug)]
pub struct YouTubeChannel {
	pub id: String,
	pub title: String,
	pub avatar_url: String,
}

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct YouTubeArticleData {
	pub id: String,
	pub creation_time: Date,
	pub title: String,
	//pub description: String,
	pub thumbnail: ArticleMedia,
	pub channel: YouTubeChannel,
	#[derivative(Debug = "ignore")]
	pub raw_json: serde_json::Value,
	pub marked_as_read: bool,
	pub hidden: bool,
}

impl ArticleData for YouTubeArticleData {
	fn service(&self) -> &'static str { SERVICE_INFO.name }

	fn id(&self) -> String { self.id.clone() }

	//TODO sortable_id → Option
	fn sortable_id(&self) -> u64 { 0 }

	fn creation_time(&self) -> Date {
		self.creation_time.clone()
	}

	fn text(&self) -> String {
		self.title.clone()
	}

	fn author_name(&self) -> String {
		self.channel.title.clone()
	}

	fn author_avatar_url(&self) -> String {
		self.channel.avatar_url.clone()
	}

	fn author_url(&self) -> String {
		format!("https://www.youtube.com/channel/{}", self.channel.id)
	}

	fn media(&self) -> Vec<ArticleMedia> {
		vec![self.thumbnail.clone()]
	}

	fn url(&self) -> String {
		format!("https://www.youtube.com/watch?v={}", self.id)
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

	fn media_loaded(&mut self, _index: usize) {}

	fn json(&self) -> Value {
		self.raw_json.clone()
	}
}

impl YouTubeArticleData {
	//TODO macro this
	pub fn update(&mut self, new: &Ref<YouTubeArticleData>) {
		self.id = new.id.clone();
		self.creation_time = new.creation_time.clone();
		self.title = new.title.clone();
		//self.description = new.description.clone();
		self.channel = new.channel.clone();
	}
}

impl From<(PlaylistItem, serde_json::Value, &ServiceStorage)> for YouTubeArticleData {
	fn from((item, raw_json, storage): (PlaylistItem, serde_json::Value, &ServiceStorage)) -> Self {
		let thumbnail = item.snippet.thumbnails.standard
			.or(item.snippet.thumbnails.maxres)
			.or(item.snippet.thumbnails.high)
			.or(item.snippet.thumbnails.medium)
			.or(item.snippet.thumbnails.default);
		YouTubeArticleData {
			id: item.snippet.resource_id.video_id.clone(),
			creation_time: Date::new(&JsValue::from_str(&item.snippet.published_at)),
			title: item.snippet.title.clone(),
			//description: item.snippet.description.clone(),
			thumbnail: ArticleMedia {
				media_type: MediaType::Image,
				//TODO std::mem::take(t.url)?
				src: thumbnail.as_ref().map(|t| t.url.clone()).unwrap_or_default(),
				ratio: thumbnail.map(|t| ValidRatio::new_u16(
					NonZeroU16::new(t.width).unwrap(),
					NonZeroU16::new(t.height).unwrap(),
				)).unwrap_or_else(|| ValidRatio::one()),
				queue_load_info: MediaQueueInfo::DirectLoad,
			},
			channel: YouTubeChannel {
				id: item.snippet.channel_id,
				title: item.snippet.channel_title,
				avatar_url: "".to_owned(),
			},
			raw_json,
			//TODO Abstract get_service_storage to ArticleData?
			marked_as_read: storage.session.articles_marked_as_read.contains(&item.snippet.resource_id.video_id),
			hidden: storage.local.hidden_articles.contains(&item.snippet.resource_id.video_id),
		}
	}
}

#[derive(Deserialize)]
pub struct PlaylistItem {
	//#[serde(rename = "contentDetails")]
	//content_details: ContentDetails,
	//etag: String,
	//id: String,
	//kind: String,
	snippet: PlaylistItemSnippet,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlaylistItemSnippet {
	channel_id: String,
	channel_title: String,
	//description: String,
	//liveBroadcastContent: null
	published_at: String,
	resource_id: PlaylistItemResourceId,
	thumbnails: Thumbnails,
	title: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlaylistItemResourceId {
	//kind: String, TODO make it an enum with kind as the tag
	video_id: String,
}

#[derive(Deserialize)]
struct Thumbnails {
	#[serde(default)]
	default: Option<Thumbnail>,
	#[serde(default)]
	high: Option<Thumbnail>,
	#[serde(default)]
	maxres: Option<Thumbnail>,
	#[serde(default)]
	medium: Option<Thumbnail>,
	#[serde(default)]
	standard: Option<Thumbnail>,
}

#[derive(Deserialize)]
struct Thumbnail {
	height: u16,
	url: String,
	width: u16,
}