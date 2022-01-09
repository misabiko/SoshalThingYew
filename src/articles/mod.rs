use std::rc::Weak;
use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter};
use std::num::{NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8};
use js_sys::Date;
use serde::{Serialize, Deserialize};
use yew::prelude::*;

pub mod component;
pub mod fetch_agent;
pub mod media_load_queue;
mod social;
mod gallery;

pub use component::ArticleComponent;
pub use crate::articles::social::SocialArticle;
pub use crate::articles::gallery::GalleryArticle;

#[derive(Clone, Debug, PartialEq)]
pub enum ArticleRefType<Pointer = Weak<RefCell<dyn ArticleData>>> {
	NoRef,
	Repost(Pointer),
	Quote(Pointer),
	QuoteRepost(Pointer, Pointer),
}

impl ArticleRefType<Box<dyn ArticleData>> {
	pub fn clone_data(&self) -> Self {
		match self {
			ArticleRefType::NoRef => ArticleRefType::NoRef,
			ArticleRefType::Repost(a) => ArticleRefType::Repost(a.clone_data()),
			ArticleRefType::Quote(a) => ArticleRefType::Quote(a.clone_data()),
			ArticleRefType::QuoteRepost(a, q) => ArticleRefType::QuoteRepost(a.clone_data(), q.clone_data()),
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ValidRatio(f32);

macro_rules! new_u {
	($name:ident, $nonzero:ty) => {
		pub fn $name(width: $nonzero, height: $nonzero) -> Self {
			Self(height.get() as f32 / width.get() as f32)
		}
	}
}

impl ValidRatio {
	new_u!{new_u8, NonZeroU8}
	new_u!{new_u16, NonZeroU16}
	new_u!{new_u32, NonZeroU32}
	new_u!{new_u64, NonZeroU64}

	pub fn one() -> Self {
		ValidRatio(1.0)
	}

	pub fn get(&self) -> &f32 {
		&self.0
	}
}

impl Into<f32> for ValidRatio {
	fn into(self) -> f32 {
		self.0
	}
}

impl Display for ValidRatio {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.0))
	}
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ArticleMedia {
	pub src: String,
	pub ratio: ValidRatio,
	pub queue_load_info: MediaQueueInfo,
	pub media_type: MediaType,
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum MediaType {
	Image,
	Video,
	VideoGif,
	Gif,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum MediaQueueInfo {
	DirectLoad,
	Thumbnail,
	LazyLoad  {
		thumbnail: Option<(String, f32)>,
		loaded: bool,
	}
}

pub trait ArticleData : Debug {
	fn service(&self) -> &'static str;
	fn id(&self) -> String;
	fn sortable_id(&self) -> usize;
	fn index(&self) -> usize { self.sortable_id() }	//TODO Use per-service sort methods
	fn creation_time(&self) -> Date;
	fn text(&self) -> String;
	fn author_name(&self) -> String;
	fn author_username(&self) -> String { self.author_name() }
	fn author_avatar_url(&self) -> String;
	fn author_url(&self) -> String;
	fn like_count(&self) -> u32 { 0 }
	fn repost_count(&self) -> u32 { 0 }
	fn liked(&self) -> bool { false }
	fn reposted(&self) -> bool { false }
	fn media(&self) -> Vec<ArticleMedia>;
	fn json(&self) -> serde_json::Value { serde_json::Value::Null }
	fn referenced_article(&self) -> ArticleRefType { ArticleRefType::NoRef }
	fn url(&self) -> String;
	fn marked_as_read(&self) -> bool;
	fn set_marked_as_read(&mut self, value: bool);
	fn hidden(&self) -> bool;
	fn set_hidden(&mut self, value: bool);
	fn is_fully_fetched(&self) -> &bool { &true }
	fn clone_data(&self) -> Box<dyn ArticleData>;
	fn media_loaded(&mut self, index: usize);
	fn view_text(&self) -> Html {
		html! { { self.text() } }
	}
}

impl PartialEq<dyn ArticleData> for dyn ArticleData {
	fn eq(&self, other: &dyn ArticleData) -> bool {
		self.id() == other.id() &&
			self.text() == other.text() &&
			self.creation_time() == other.creation_time() &&
			self.author_username() == other.author_username() &&
			self.author_name() == other.author_name() &&
			self.author_avatar_url() == other.author_avatar_url() &&
			self.author_url() == other.author_url() &&
			self.like_count() == other.like_count() &&
			self.liked() == other.liked() &&
			self.repost_count() == other.repost_count() &&
			self.reposted() == other.reposted() &&
			self.media() == other.media() &&
			self.marked_as_read() == other.marked_as_read() &&
			self.hidden() == other.hidden() &&
			self.is_fully_fetched() == other.is_fully_fetched()
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ArticleView {
	Social,
	Gallery
}

impl ArticleView {
	pub fn name(&self) -> &'static str {
		match self {
			ArticleView::Social => "Social",
			ArticleView::Gallery => "Gallery",
		}
	}
}

pub fn actual_article(article: &Weak<RefCell<dyn ArticleData>>) -> Weak<RefCell<dyn ArticleData>> {
	if let Some(strong) = article.upgrade() {
		let borrow = strong.borrow();

		match borrow.referenced_article() {
			ArticleRefType::NoRef | ArticleRefType::Quote(_) => article.clone(),
			ArticleRefType::Repost(a) | ArticleRefType::QuoteRepost(a, _) => a.clone()
		}
	}else {
		log::warn!("Couldn't unwrap article.");
		article.clone()
	}
}