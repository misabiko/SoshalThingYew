use std::cell::Ref;
use js_sys::Date;
use serde::{Serialize, Deserialize};

use crate::articles::{ArticleData, ArticleMedia, MediaQueueInfo};

#[derive(Clone, Debug)]
pub struct PixivArticleData {
	pub id: u32,
	pub creation_time: Date,
	pub media: ArticleMedia,
	pub title: String,
	pub author_name: String,
	pub author_id: u32,
	pub author_avatar_url: String,
	pub marked_as_read: bool,
	pub hidden: bool,
	pub is_fully_fetched: bool,
	pub raw_json: serde_json::Value,
	pub like_count: u32,
	pub liked: bool,
	pub bookmark_count: u32,
	pub bookmarked: bool,
}

impl ArticleData for PixivArticleData {
	fn service(&self) -> &'static str {
		"Pixiv"
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
		self.title.clone()
	}

	fn author_username(&self) -> String {
		self.author_id.clone().to_string()
	}

	fn author_name(&self) -> String {
		self.author_name.clone()
	}

	fn author_avatar_url(&self) -> String {
		self.author_avatar_url.clone()
	}

	fn author_url(&self) -> String {
		format!("https://www.pixiv.net/en/users/{}", &self.author_id)
	}

	fn media(&self) -> Vec<ArticleMedia> {
		vec![self.media.clone()]
	}

	fn json(&self) -> serde_json::Value {
		self.raw_json.clone()
	}

	fn url(&self) -> String {
		format!("https://www.pixiv.net/en/artworks/{}", &self.id)
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

	fn is_fully_fetched(&self) -> &bool { &self.is_fully_fetched }

	fn like_count(&self) -> u32 {
		self.like_count
	}

	fn liked(&self) -> bool {
		self.liked
	}

	fn repost_count(&self) -> u32 {
		self.bookmark_count
	}

	fn reposted(&self) -> bool {
		self.bookmarked
	}

	fn clone_data(&self) -> Box<dyn ArticleData> {
		Box::new(self.clone())
	}

	fn media_loaded(&mut self, _index: usize) {
		if let MediaQueueInfo::LazyLoad { loaded, .. } = &mut self.media.queue_load_info {
			*loaded = true;
		}
	}
}

impl PixivArticleData {
	pub fn update(&mut self, new: &Ref<PixivArticleData>) {
		self.media = new.media.clone();
		self.title = new.title.clone();
		self.is_fully_fetched = self.is_fully_fetched || *new.is_fully_fetched();
		match &new.raw_json {
			serde_json::Value::Null => {}
			new_json => self.raw_json = new_json.clone(),
		};

		self.like_count = new.like_count;
		self.liked = new.liked;
		self.bookmark_count = new.bookmark_count;
		self.bookmarked = new.bookmarked;
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PixivArticleCached {
	pub id: u32,
	pub media: ArticleMedia,
	pub author_avatar_url: String,
}

impl From<&Ref<'_, PixivArticleData>> for PixivArticleCached {
	fn from(article: &Ref<'_, PixivArticleData>) -> Self {
		Self {
			id: article.id.clone(),
			media: article.media.clone(),
			author_avatar_url: article.author_avatar_url.clone(),
		}
	}
}