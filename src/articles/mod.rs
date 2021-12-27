use std::rc::Weak;
use std::cell::{RefCell, Ref};
use yew::prelude::*;
use js_sys::Date;

pub mod fetch_agent;
mod social;
mod gallery;

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
	fn creation_time(&self) -> Date;
	fn text(&self) -> String;
	fn author_username(&self) -> String;
	fn author_name(&self) -> String;
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
	fn update(&mut self, new: &Ref<dyn ArticleData>);
	fn marked_as_read(&self) -> bool;
	fn set_marked_as_read(&mut self, value: bool);
	fn hidden(&self) -> bool;
	fn set_hidden(&mut self, value: bool);
	fn is_fully_fetched(&self) -> &bool { &true }
}

impl PartialEq<dyn ArticleData> for dyn ArticleData {
	fn eq(&self, other: &dyn ArticleData) -> bool {
		self.id() == other.id() &&
			self.text() == other.text() &&
			self.author_username() == other.author_username() &&
			self.author_name() == other.author_name() &&
			self.author_avatar_url() == other.author_avatar_url() &&
			self.author_url() == other.author_url() &&
			self.media() == other.media()
	}
}

#[derive(Properties, Clone)]
pub struct Props {
	pub data: Weak<RefCell<dyn ArticleData>>,
	#[prop_or_default]
	pub compact: bool,
	#[prop_or_default]
	pub style: Option<String>,
	#[prop_or_default]
	pub animated_as_gifs: bool,
	#[prop_or_default]
	pub hide_text: bool,
}

impl PartialEq<Props> for Props {
	fn eq(&self, other: &Props) -> bool {
		self.compact == other.compact &&
		self.animated_as_gifs == other.animated_as_gifs &&
		self.hide_text == other.hide_text &&
			self.style == other.style &&
			Weak::ptr_eq(&self.data, &other.data)
	}
}

#[derive(Clone, PartialEq, Eq)]
pub enum ArticleComponent {
	Social,
	Gallery
}

impl ArticleComponent {
	pub fn name(&self) -> &'static str {
		match self {
			ArticleComponent::Social => "Social",
			ArticleComponent::Gallery => "Gallery",
		}
	}
}

pub fn view_article(component: &ArticleComponent, compact: bool, animated_as_gifs: bool, hide_text: bool, style: Option<String>, article: Weak<RefCell<dyn ArticleData>>) -> Html {
	match component {
		ArticleComponent::Social => html! {
			<SocialArticle {compact} {animated_as_gifs} {hide_text} {style} data={article.clone()}/>
		},
		ArticleComponent::Gallery => html! {
			<GalleryArticle {compact} {animated_as_gifs} {hide_text} {style} data={article.clone()}/>
		},
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