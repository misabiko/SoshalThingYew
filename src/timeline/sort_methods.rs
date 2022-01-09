use std::rc::Weak;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use serde::{Serialize, Deserialize};

use crate::articles::{actual_article, ArticleData};

//TODO Check for cases where Copy is derivable
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum SortMethod {
	Id,
	Index,
	Date,
	Likes,
	Reposts,
}

impl SortMethod {
	pub fn iter() -> impl ExactSizeIterator<Item = &'static SortMethod> {
		[
			SortMethod::Id,
			SortMethod::Index,
			SortMethod::Date,
			SortMethod::Likes,
			SortMethod::Reposts,
		].iter()
	}

	pub fn direction_label(&self, reversed: bool) -> &'static str {
		match self {
			SortMethod::Date => if reversed { "Reverse chronological" } else { "Chronological" },
			SortMethod::Likes | SortMethod::Reposts => if reversed { "Descending" } else { "Ascending" },
			_ => if reversed { "Descending" } else { "Ascending" },
		}
	}

	//TODO Unit test sort methods
	pub fn compare(&self, a: &Weak<RefCell<dyn ArticleData>>, b: &Weak<RefCell<dyn ArticleData>>) -> std::cmp::Ordering {
		match self {
			SortMethod::Id => {
				let a = a.upgrade().map(|s| s.borrow().sortable_id()).unwrap_or_default();
				let b = b.upgrade().map(|s| s.borrow().sortable_id()).unwrap_or_default();
				a.partial_cmp(&b).unwrap()
			},
			SortMethod::Index => {
				let a = a.upgrade().map(|s| s.borrow().index()).unwrap_or_default();
				let b = b.upgrade().map(|s| s.borrow().index()).unwrap_or_default();
				a.partial_cmp(&b).unwrap()
			}
			SortMethod::Date => {
				let a = a.upgrade().map(|s| s.borrow().creation_time()).map(|d| d.get_time()).unwrap_or(0.0);
				let b = b.upgrade().map(|s| s.borrow().creation_time()).map(|d| d.get_time()).unwrap_or(0.0);
				a.partial_cmp(&b).unwrap()
			}
			SortMethod::Likes => {
				let (a, b) = (actual_article(&a), actual_article(&b));
				let a = a.upgrade().map(|s| s.borrow().like_count()).unwrap_or_default();
				let b = b.upgrade().map(|s| s.borrow().like_count()).unwrap_or_default();
				a.partial_cmp(&b).unwrap()
			}
			SortMethod::Reposts => {
				let (a, b) = (actual_article(&a), actual_article(&b));
				let a = a.upgrade().map(|s| s.borrow().repost_count()).unwrap_or_default();
				let b = b.upgrade().map(|s| s.borrow().repost_count()).unwrap_or_default();
				a.partial_cmp(&b).unwrap()
			}
		}
	}
}

impl Display for SortMethod {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			SortMethod::Id => f.write_str("Id"),
			SortMethod::Index => f.write_str("Index"),
			SortMethod::Date => f.write_str("Date"),
			SortMethod::Likes => f.write_str("Likes"),
			SortMethod::Reposts => f.write_str("Reposts"),
		}
	}
}

impl Default for SortMethod {
	fn default() -> Self {
		SortMethod::Id
	}
}

//TODO use sort method
pub fn sort_by_id(a: &Weak<RefCell<dyn ArticleData>>, b: &Weak<RefCell<dyn ArticleData>>) -> std::cmp::Ordering {
	let a_id = a.upgrade().map(|s| s.borrow().id()).unwrap_or("0".to_owned());
	let b_id = b.upgrade().map(|s| s.borrow().id()).unwrap_or("0".to_owned());
	b_id.partial_cmp(&a_id).unwrap()
}