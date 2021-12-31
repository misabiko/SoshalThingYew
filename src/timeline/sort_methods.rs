use std::rc::Weak;
use std::cell::RefCell;

use crate::articles::{actual_article, ArticleData};

pub struct SortMethod {
	pub name: String,
	pub compare: fn(a: &Weak<RefCell<dyn ArticleData>>, b: &Weak<RefCell<dyn ArticleData>>) -> std::cmp::Ordering,
}

pub fn default_sort_methods() -> Vec<SortMethod> {
	vec![SortMethod {
		name: "Id".to_owned(),
		compare: |a, b| {
			let a = a.upgrade().and_then(|s| s.borrow().id().parse::<u32>().ok()).unwrap_or_default();
			let b = b.upgrade().and_then(|s| s.borrow().id().parse::<u32>().ok()).unwrap_or_default();
			b.partial_cmp(&a).unwrap()
		},
	},SortMethod {
		name: "Date".to_owned(),
		compare: |a, b| {
			let a = a.upgrade().map(|s| s.borrow().creation_time()).map(|d| d.get_time()).unwrap_or(0.0);
			let b = b.upgrade().map(|s| s.borrow().creation_time()).map(|d| d.get_time()).unwrap_or(0.0);
			b.partial_cmp(&a).unwrap()
		},
	},SortMethod {
		name: "Likes".to_owned(),
		compare: |a, b| {
			let (a, b) = (actual_article(&a), actual_article(&b));
			let a = a.upgrade().map(|s| s.borrow().like_count()).unwrap_or_default();
			let b = b.upgrade().map(|s| s.borrow().like_count()).unwrap_or_default();
			b.partial_cmp(&a).unwrap()
		},
	},SortMethod {
		name: "Reposts".to_owned(),
		compare: |a, b| {
			let (a, b) = (actual_article(&a), actual_article(&b));
			let a = a.upgrade().map(|s| s.borrow().repost_count()).unwrap_or_default();
			let b = b.upgrade().map(|s| s.borrow().repost_count()).unwrap_or_default();
			b.partial_cmp(&a).unwrap()
		},
	},]
}

pub fn sort_by_id(a: &Weak<RefCell<dyn ArticleData>>, b: &Weak<RefCell<dyn ArticleData>>) -> std::cmp::Ordering {
	let a_id = a.upgrade().map(|s| s.borrow().id()).unwrap_or("0".to_owned());
	let b_id = b.upgrade().map(|s| s.borrow().id()).unwrap_or("0".to_owned());
	b_id.partial_cmp(&a_id).unwrap()
}