use std::rc::Weak;
use std::cell::RefCell;
use js_sys::Date;

pub mod component;
pub mod fetch_agent;
mod social;
mod gallery;

pub use component::ArticleComponent;
pub use crate::articles::social::SocialArticle;
pub use crate::articles::gallery::GalleryArticle;

#[derive(Clone)]
pub enum ArticleRefType<Pointer = Weak<RefCell<dyn ArticleData>>> {
	NoRef,
	Repost(Pointer),
	Quote(Pointer),
	QuoteRepost(Pointer, Pointer),
}

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum ArticleMedia {
	Image(String, f32),
	Video(String, f32),
	VideoGif(String, f32),
	Gif(String, f32),
}

pub trait ArticleData {
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

#[derive(Clone, PartialEq, Eq)]
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