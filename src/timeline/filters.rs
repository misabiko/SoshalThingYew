use std::rc::Weak;
use std::cell::RefCell;

use crate::articles::ArticleData;

pub struct Filter {
	pub predicate: fn(&Weak<RefCell<dyn ArticleData>>) -> bool,
	pub enabled: bool,
	pub inverted: bool,
}

impl Filter {
	fn new(predicate: fn(&Weak<RefCell<dyn ArticleData>>) -> bool) -> Self {
		Self {
			predicate,
			enabled: true,
			inverted: false,
		}
	}
}

pub fn default_filters() -> Vec<Filter> {
	vec![
		Filter::new(|a| {
			match a.upgrade() {
				Some(strong) => strong.borrow().media().len() > 0,
				None => false,
			}
		})
	]
}