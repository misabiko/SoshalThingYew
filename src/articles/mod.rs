use std::rc::{Rc, Weak};
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

pub trait ArticleData : Debug {
	//TODO type Id;

	fn service(&self) -> &'static str;
	fn id(&self) -> String;
	fn sortable_id(&self) -> u64;
	fn index(&self) -> u64 { self.sortable_id() }	//TODO Use per-service sort methods
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
	fn unfetched_references(&self) -> Vec<UnfetchedArticleRef> { Vec::new() }
	fn referenced_articles(&self) -> Vec<ArticleRefType> { Vec::new() }
	fn actual_article_index(&self) -> Option<usize> { None }
	fn actual_article(&self) -> Option<ArticleWeak> { None }
	fn url(&self) -> String;
	fn marked_as_read(&self) -> bool;
	fn set_marked_as_read(&mut self, value: bool);
	fn hidden(&self) -> bool;
	fn set_hidden(&mut self, value: bool);
	fn is_fully_fetched(&self) -> &bool { &true }
	fn clone_data(&self) -> ArticleBox;
	fn media_loaded(&mut self, index: usize);
	fn view_text(&self) -> Html {
		html! { { self.text() } }
	}
}

//type ArticlePtr<Pointer> = Pointer<RefCell<dyn ArticleData>>;
pub type ArticleRc<A = dyn ArticleData> = Rc<RefCell<A>>;
pub type ArticleWeak<A = dyn ArticleData> = Weak<RefCell<A>>;
pub type ArticleBox<A = dyn ArticleData> = Box<A>;

#[derive(Clone, Debug, PartialEq)]
pub enum ArticleRefType<Pointer = ArticleWeak> {
	Reposted(Pointer),
	Quote(Pointer),
	RepostedQuote(Pointer, Pointer),
	Reply(Pointer),
}

impl ArticleRefType<ArticleBox> {
	pub fn clone_data(&self) -> Self {
		match self {
			ArticleRefType::Reposted(a) => ArticleRefType::Reposted(a.clone_data()),
			ArticleRefType::Quote(a) => ArticleRefType::Quote(a.clone_data()),
			ArticleRefType::RepostedQuote(a, q) => ArticleRefType::RepostedQuote(a.clone_data(), q.clone_data()),
			ArticleRefType::Reply(a) => ArticleRefType::Reply(a.clone_data()),
		}
	}
}

//Might be a bit too biased toward Twitter ^^
#[derive(Clone, Debug, PartialEq)]
pub enum UnfetchedArticleRef {
	Unused(String),	//Remove this when we add a second variant
	ReplyToUser(String),
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

pub fn weak_actual_article(article: &ArticleWeak) -> ArticleWeak {
	let strong = article.upgrade().unwrap();
	let borrow = strong.borrow();

	match borrow.actual_article() {
		Some(a) => a,
		None => article.clone(),
	}
}