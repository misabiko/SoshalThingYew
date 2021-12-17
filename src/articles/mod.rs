use std::rc::{Rc, Weak};
use yew::prelude::*;
use js_sys::Date;

pub mod social;
pub mod gallery;

use crate::articles::social::SocialArticle;
use crate::articles::gallery::GalleryArticle;

pub trait ArticleData {
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
	fn referenced_article(&self) -> Option<Weak<dyn ArticleData>> { None }
	fn url(&self) -> String;
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
	pub data: Rc<dyn ArticleData>,
}

impl PartialEq<Props> for Props {
	fn eq(&self, other: &Props) -> bool {
		self.compact == other.compact &&
			self.style == other.style &&
			&self.data == &other.data
	}
}

pub fn sort_by_id(a: &Rc<dyn ArticleData>, b: &Rc<dyn ArticleData>) -> std::cmp::Ordering {
	b.id().partial_cmp(&a.id()).unwrap()
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

pub fn view_article(component: &ArticleComponent, article: Rc<dyn ArticleData>) -> Html {
	match component {
		ArticleComponent::Social => html! {
			<SocialArticle compact={false} data={article.clone()}/>
		},
		ArticleComponent::Gallery => html! {
			<GalleryArticle compact={false} data={article.clone()}/>
		},
	}
}