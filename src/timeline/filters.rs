use std::rc::Weak;
use std::cell::{Ref, RefCell};
use serde::{Serialize, Deserialize};

use crate::articles::{ArticleData, ArticleMedia, ArticleRefType, MediaType};

pub type FilterPredicate = fn(&Weak<RefCell<dyn ArticleData>>, inverted: &bool) -> bool;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Filter {
	Media,
	Animated,
	NotMarkedAsRead,
	NotHidden,
	Liked,
	Reposted,
	PlainTweet,
	Repost,
	Quote,
}

impl Filter {
	pub fn name(&self, inverted: bool) -> &'static str {
		if inverted {
			match self {
				Filter::Media => "Without Media",
				Filter::Animated => "Not Animated",
				Filter::NotMarkedAsRead => "Marked as read",
				Filter::NotHidden => "Hidden",
				Filter::Liked => "Not Liked",
				Filter::Reposted => "Not Reposted",
				Filter::PlainTweet => "Not Plain Tweet",
				Filter::Repost => "Not Repost",
				Filter::Quote => "No Quote",
			}
		}else {
			match self {
				Filter::Media => "Has Media",
				Filter::Animated => "Animated",
				Filter::NotMarkedAsRead => "Not marked as read",
				Filter::NotHidden => "Not hidden",
				Filter::Liked => "Liked",
				Filter::Reposted => "Reposted",
				Filter::PlainTweet => "Plain Tweet",
				Filter::Repost => "Repost",
				Filter::Quote => "Has Quote",
			}
		}
	}
	pub fn iter() -> impl ExactSizeIterator<Item = &'static Filter> {
		[
			Filter::Media,
			Filter::Animated,
			Filter::NotMarkedAsRead,
			Filter::NotHidden,
			Filter::Liked,
			Filter::Reposted,
			Filter::PlainTweet,
			Filter::Repost,
			Filter::Quote,
		].iter()
	}

	pub fn filter(&self, article: &Ref<dyn ArticleData>) -> bool {
		match self {
			Filter::Media => {
				match article.referenced_article() {
					ArticleRefType::NoRef => !article.media().is_empty(),
					ArticleRefType::Repost(a) => a.upgrade().map(|r| !r.borrow().media().is_empty()).unwrap_or(false),
					ArticleRefType::Quote(a) => (a.upgrade().map(|r| !r.borrow().media().is_empty()).unwrap_or(false) || !article.media().is_empty()),
					ArticleRefType::QuoteRepost(a, q) => (q.upgrade().map(|r| !r.borrow().media().is_empty()).unwrap_or(false) || a.upgrade().map(|r| !r.borrow().media().is_empty()).unwrap_or(false) || !article.media().is_empty()),
				}
			}
			Filter::Animated => {
				match article.referenced_article() {
					ArticleRefType::NoRef => article.media().iter().any(|m| is_animated(m)),
					ArticleRefType::Repost(a) => a.upgrade().map(|r| r.borrow().media().iter().any(|m| is_animated(m))).unwrap_or(false),
					ArticleRefType::Quote(a) => (a.upgrade().map(|r| r.borrow().media().iter().any(|m| is_animated(m))).unwrap_or(false) || (article.media().iter().any(|m| is_animated(m)))),
					ArticleRefType::QuoteRepost(a, q) => (q.upgrade().map(|r| r.borrow().media().iter().any(|m| is_animated(m))).unwrap_or(false) || a.upgrade().map(|r| r.borrow().media().iter().any(|m| is_animated(m))).unwrap_or(false) || (article.media().iter().any(|m| is_animated(m)))),
				}
			},
			Filter::NotMarkedAsRead => {
				match article.referenced_article() {
					ArticleRefType::NoRef => (!article.marked_as_read()),
					ArticleRefType::Repost(a) | ArticleRefType::Quote(a)
					=> (a.upgrade().map(|r| !r.borrow().marked_as_read()).unwrap_or(false) && !article.marked_as_read()),
					ArticleRefType::QuoteRepost(a, q)
					=> (q.upgrade().map(|r| !r.borrow().marked_as_read()).unwrap_or(false) && a.upgrade().map(|r| !r.borrow().marked_as_read()).unwrap_or(false) && !article.marked_as_read()),
				}
			},
			Filter::NotHidden => {
				match article.referenced_article() {
					ArticleRefType::NoRef => !article.hidden(),
					ArticleRefType::Repost(a) | ArticleRefType::Quote(a)
					=> a.upgrade().map(|r| !r.borrow().hidden()).unwrap_or(false) && !article.hidden(),
					ArticleRefType::QuoteRepost(a, q)
					=> q.upgrade().map(|r| !r.borrow().hidden()).unwrap_or(false) && a.upgrade().map(|r| !r.borrow().hidden()).unwrap_or(false) && !article.hidden(),
				}
			}
			Filter::Liked => {
				match article.referenced_article() {
					ArticleRefType::NoRef => article.liked(),
					ArticleRefType::Repost(a) | ArticleRefType::Quote(a)
					=> a.upgrade().map(|r| r.borrow().liked()).unwrap_or(false) || article.liked(),
					ArticleRefType::QuoteRepost(a, q)
					=> q.upgrade().map(|r| r.borrow().liked()).unwrap_or(false) || a.upgrade().map(|r| r.borrow().liked()).unwrap_or(false) || article.liked(),
				}
			}
			Filter::Reposted => {
				match article.referenced_article() {
					ArticleRefType::NoRef => article.reposted(),
					ArticleRefType::Repost(a) | ArticleRefType::Quote(a)
					=> a.upgrade().map(|r| r.borrow().reposted()).unwrap_or(false) || article.reposted(),
					ArticleRefType::QuoteRepost(a, q)
					=> q.upgrade().map(|r| r.borrow().reposted()).unwrap_or(false) || a.upgrade().map(|r| r.borrow().reposted()).unwrap_or(false) || article.reposted(),
				}
			}
			Filter::PlainTweet => {
				if let ArticleRefType::NoRef = article.referenced_article() {
					true
				}else {
					false
				}
			}
			Filter::Repost => {
				match article.referenced_article() {
					ArticleRefType::Repost(_) | ArticleRefType::QuoteRepost(_, _) => true,
					ArticleRefType::NoRef | ArticleRefType::Quote(_) => false,
				}
			}
			Filter::Quote => {
				match article.referenced_article() {
					ArticleRefType::Quote(_) | ArticleRefType::QuoteRepost(_, _) => true,
					ArticleRefType::NoRef | ArticleRefType::Repost(_) => false,
				}
			}
		}
	}
}

//TODO Add Eq where it makes sense
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FilterInstance {
	pub filter: Filter,
	pub enabled: bool,
	pub inverted: bool,
}

impl FilterInstance {
	pub fn new(filter: Filter) -> Self {
		Self {
			filter,
			enabled: true,
			inverted: false,
		}
	}

	pub fn new_disabled(filter: Filter) -> Self {
		Self {
			filter,
			enabled: false,
			inverted: false,
		}
	}
}

fn is_animated(media: &ArticleMedia) -> bool {
	match media.media_type {
		MediaType::Video | MediaType::VideoGif | MediaType::Gif => true,
		MediaType::Image => false,
	}
}