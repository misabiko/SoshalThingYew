use std::rc::Weak;
use std::cell::RefCell;

use crate::articles::{ArticleData, ArticleMedia, ArticleRefType};

pub type FilterPredicate = fn(&Weak<RefCell<dyn ArticleData>>, inverted: bool) -> bool;

pub struct Filter {
	pub name: String,
	pub predicate: FilterPredicate,
	pub enabled: bool,
	pub inverted: bool,
}

impl Filter {
	fn new(name: String, predicate: FilterPredicate) -> Self {
		Self {
			name,
			predicate,
			enabled: true,
			inverted: false,
		}
	}

	fn new_disabled(name: String, predicate: FilterPredicate) -> Self {
		Self {
			name,
			predicate,
			enabled: false,
			inverted: false,
		}
	}
}

fn is_animated(media: &ArticleMedia) -> bool {
	match media {
		ArticleMedia::Video(_, _) | ArticleMedia::Gif(_, _) => true,
		ArticleMedia::Image(_, _) => false,
	}
}

pub fn default_filters() -> Vec<Filter> {
	vec![
		Filter::new_disabled(
			"Media".to_owned(),
			|a, inverted| {
				match a.upgrade() {
					Some(strong) => {
						let borrow = strong.borrow();
						(match borrow.referenced_article() {
							ArticleRefType::NoRef => borrow.media().iter().any(|m| is_animated(m)),
							ArticleRefType::Repost(a) => a.upgrade().map(|r| r.borrow().media().len() > 0).unwrap_or(false),
							ArticleRefType::Quote(a) => (a.upgrade().map(|r| r.borrow().media().len() > 0).unwrap_or(false) || (borrow.media().len() > 0)),
							ArticleRefType::QuoteRepost(a, q) => (q.upgrade().map(|r| r.borrow().media().len() > 0).unwrap_or(false) || a.upgrade().map(|r| r.borrow().media().len() > 0).unwrap_or(false) || (borrow.media().len() > 0)),
						}) != inverted
					},
					None => false,
				}
		}),
		Filter::new_disabled(
			"Animated".to_owned(),
			|a, inverted| {
				match a.upgrade() {
					Some(strong) => {
						let borrow = strong.borrow();
						(match borrow.referenced_article() {
							ArticleRefType::NoRef => borrow.media().iter().any(|m| is_animated(m)),
							ArticleRefType::Repost(a) => a.upgrade().map(|r| r.borrow().media().iter().any(|m| is_animated(m))).unwrap_or(false),
							ArticleRefType::Quote(a) => (a.upgrade().map(|r| r.borrow().media().iter().any(|m| is_animated(m))).unwrap_or(false) || (borrow.media().iter().any(|m| is_animated(m)))),
							ArticleRefType::QuoteRepost(a, q) => (q.upgrade().map(|r| r.borrow().media().iter().any(|m| is_animated(m))).unwrap_or(false) || a.upgrade().map(|r| r.borrow().media().iter().any(|m| is_animated(m))).unwrap_or(false) || (borrow.media().iter().any(|m| is_animated(m)))),
						}) != inverted
					},
					None => false,
				}
			}),
		Filter::new(
			"Not marked as read".to_owned(),
			|a, inverted| {
				match a.upgrade() {
					Some(strong) => {
						let borrow = strong.borrow();
						(match borrow.referenced_article() {
							ArticleRefType::NoRef => (!borrow.marked_as_read()),
							ArticleRefType::Repost(a) | ArticleRefType::Quote(a)
								=> (a.upgrade().map(|r| !r.borrow().marked_as_read()).unwrap_or(false) && !borrow.marked_as_read()),
							ArticleRefType::QuoteRepost(a, q)
							=> (q.upgrade().map(|r| !r.borrow().marked_as_read()).unwrap_or(false) && a.upgrade().map(|r| !r.borrow().marked_as_read()).unwrap_or(false) && !borrow.marked_as_read()),
						}) != inverted
					},
					None => false,
				}
			}),
		Filter::new(
			"Not hidden".to_owned(),
			|a, inverted| {
				match a.upgrade() {
					Some(strong) => {
						let borrow = strong.borrow();
						(match borrow.referenced_article() {
							ArticleRefType::NoRef => !borrow.hidden(),
							ArticleRefType::Repost(a) | ArticleRefType::Quote(a)
								=> a.upgrade().map(|r| !r.borrow().hidden()).unwrap_or(false) && !borrow.hidden(),
							ArticleRefType::QuoteRepost(a, q)
							=> q.upgrade().map(|r| !r.borrow().hidden()).unwrap_or(false) && a.upgrade().map(|r| !r.borrow().hidden()).unwrap_or(false) && !borrow.hidden(),
						}) != inverted
					},
					None => false,
				}
			}),
		Filter::new_disabled(
			"Liked".to_owned(),
			|a, inverted| {
				match a.upgrade() {
					Some(strong) => {
						let borrow = strong.borrow();
						(match borrow.referenced_article() {
							ArticleRefType::NoRef => borrow.liked(),
							ArticleRefType::Repost(a) | ArticleRefType::Quote(a)
							=> a.upgrade().map(|r| r.borrow().liked()).unwrap_or(false) || borrow.liked(),
							ArticleRefType::QuoteRepost(a, q)
							=> q.upgrade().map(|r| r.borrow().liked()).unwrap_or(false) || a.upgrade().map(|r| r.borrow().liked()).unwrap_or(false) || borrow.liked(),
						}) != inverted
					},
					None => false,
				}
			}),
		Filter::new_disabled(
			"Reposted".to_owned(),
			|a, inverted| {
				match a.upgrade() {
					Some(strong) => {
						let borrow = strong.borrow();
						(match borrow.referenced_article() {
							ArticleRefType::NoRef => borrow.reposted(),
							ArticleRefType::Repost(a) | ArticleRefType::Quote(a)
							=> a.upgrade().map(|r| r.borrow().reposted()).unwrap_or(false) || borrow.reposted(),
							ArticleRefType::QuoteRepost(a, q)
							=> q.upgrade().map(|r| r.borrow().reposted()).unwrap_or(false) || a.upgrade().map(|r| r.borrow().reposted()).unwrap_or(false) || borrow.reposted(),
						}) != inverted
					},
					None => false,
				}
			}),
	]
}