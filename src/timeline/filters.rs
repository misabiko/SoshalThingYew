use std::rc::Weak;
use std::cell::RefCell;

use crate::articles::{ArticleData, ArticleRefType};

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
}

pub fn default_filters() -> Vec<Filter> {
	vec![
		Filter::new(
			"Media".to_owned(),
			|a, inverted| {
				match a.upgrade() {
					Some(strong) => {
						let borrow = strong.borrow();
						(match borrow.referenced_article() {
							ArticleRefType::NoRef => (borrow.media().len() > 0),
							ArticleRefType::Repost(a) => a.upgrade().map(|r| r.borrow().media().len() > 0).unwrap_or(false),
							ArticleRefType::Quote(a) => (a.upgrade().map(|r| r.borrow().media().len() > 0).unwrap_or(false) || (borrow.media().len() > 0)),
						}) != inverted
					},
					None => false,
				}
		})
	]
}