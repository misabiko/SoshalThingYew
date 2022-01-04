use std::rc::Weak;
use std::cell::RefCell;

use crate::articles::{ArticleData, ArticleMedia, ArticleRefType, MediaType};

pub type FilterPredicate = fn(&Weak<RefCell<dyn ArticleData>>, inverted: &bool) -> bool;

#[derive(Clone)]
pub struct Filter {
	pub name: String,
	pub predicate: FilterPredicate,
	pub enabled: bool,
	pub inverted: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct FilterSerialized {
	pub id: usize,
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
	match media.media_type {
		MediaType::Video | MediaType::VideoGif | MediaType::Gif => true,
		MediaType::Image => false,
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
						&(match borrow.referenced_article() {
							ArticleRefType::NoRef => !borrow.media().is_empty(),
							ArticleRefType::Repost(a) => a.upgrade().map(|r| !r.borrow().media().is_empty()).unwrap_or(false),
							ArticleRefType::Quote(a) => (a.upgrade().map(|r| !r.borrow().media().is_empty()).unwrap_or(false) || !borrow.media().is_empty()),
							ArticleRefType::QuoteRepost(a, q) => (q.upgrade().map(|r| !r.borrow().media().is_empty()).unwrap_or(false) || a.upgrade().map(|r| !r.borrow().media().is_empty()).unwrap_or(false) || !borrow.media().is_empty()),
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
						&(match borrow.referenced_article() {
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
						&(match borrow.referenced_article() {
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
						&(match borrow.referenced_article() {
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
						&(match borrow.referenced_article() {
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
						&(match borrow.referenced_article() {
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

pub fn deserialize_filters(filters: &Vec<FilterSerialized>) -> Vec<Filter> {
	//TODO Implement Filter::from<FilterSerialized>
	let default_filters = default_filters();
	filters.iter().map(|f| {
		let mut filter = default_filters[f.id].clone();
		filter.enabled = f.enabled;
		filter.inverted = f.inverted;
		filter
	}).collect()
}