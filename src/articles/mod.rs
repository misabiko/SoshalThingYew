use std::rc::Weak;
use std::cell::{RefCell, Ref};
use yew::prelude::*;
use js_sys::Date;

pub mod social;
pub mod gallery;

use crate::articles::social::SocialArticle;
use crate::articles::gallery::GalleryArticle;

#[derive(Clone)]
pub enum ArticleRefType {
	NoRef,
	Repost(Weak<RefCell<dyn ArticleData>>),
	Quote(Weak<RefCell<dyn ArticleData>>),
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
	fn like_count(&self) -> i64 { 0 }
	fn repost_count(&self) -> i64 { 0 }
	fn liked(&self) -> bool { false }
	fn reposted(&self) -> bool { false }
	fn media(&self) -> Vec<String>;
	fn json(&self) -> serde_json::Value { serde_json::Value::Null }
	fn referenced_article(&self) -> ArticleRefType { ArticleRefType::NoRef }
	fn url(&self) -> String;
	fn update(&mut self, new: &Ref<dyn ArticleData>);
	fn marked_as_read(&self) -> bool;
	fn set_marked_as_read(&mut self, value: bool);
	fn hidden(&self) -> bool;
	fn set_hidden(&mut self, value: bool);
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
	#[prop_or_default]
	pub compact: bool,
	#[prop_or_default]
	pub style: Option<String>,
	pub data: Weak<RefCell<dyn ArticleData>>,
}

impl PartialEq<Props> for Props {
	fn eq(&self, other: &Props) -> bool {
		self.compact == other.compact &&
			self.style == other.style &&
			Weak::ptr_eq(&self.data, &other.data)
	}
}

pub fn sort_by_id(a: &Weak<RefCell<dyn ArticleData>>, b: &Weak<RefCell<dyn ArticleData>>) -> std::cmp::Ordering {
	let a_id = a.upgrade().map(|s| s.borrow().id()).unwrap_or("0".to_owned());
	let b_id = b.upgrade().map(|s| s.borrow().id()).unwrap_or("0".to_owned());
	b_id.partial_cmp(&a_id).unwrap()
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

pub fn view_article(component: &ArticleComponent, article: Weak<RefCell<dyn ArticleData>>) -> Html {
	match component {
		ArticleComponent::Social => html! {
			<SocialArticle compact={false} data={article.clone()}/>
		},
		ArticleComponent::Gallery => html! {
			<GalleryArticle compact={false} data={article.clone()}/>
		},
	}
}