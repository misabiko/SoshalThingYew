use std::cell::Ref;
use js_sys::Date;

use crate::articles::{ArticleData, ArticleMedia};

#[derive(Clone, Debug)]
pub struct YoutubeChannel {
	pub name: String,
	pub id: String,
	pub avatar_url: String,
}

#[derive(Clone, Debug)]
pub struct YoutubeArticleData {
	pub id: String,
	pub creation_time: Date,
	pub description: String,
	pub channel: YoutubeChannel,
	pub marked_as_read: bool,
	pub hidden: bool,
}

impl YoutubeArticleData {
	pub fn update(&mut self, new: &Ref<YoutubeArticleData>) {
		todo!()
	}
}

impl ArticleData for YoutubeArticleData {
	fn service(&self) -> &'static str { "Youtube" }

	fn id(&self) -> String { self.id.clone() }

	//TODO sortable_id → Option
	fn sortable_id(&self) -> u64 { 0 }

	fn creation_time(&self) -> Date {
		self.creation_time.clone()
	}

	fn text(&self) -> String {
		self.description.clone()
	}

	fn author_name(&self) -> String {
		self.channel.name.clone()
	}

	fn author_avatar_url(&self) -> String {
		self.channel.avatar_url.clone()
	}

	fn author_url(&self) -> String {
		format!("https://www.youtube.com/channel/{}", self.channel.id)
	}

	fn media(&self) -> Vec<ArticleMedia> {
		Vec::new()
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
}