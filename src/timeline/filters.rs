use std::rc::Weak;
use std::cell::{Ref, RefCell};
use serde::{Serialize, Deserialize};
use yew::prelude::*;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

use crate::articles::{ArticleData, ArticleMedia, ArticleRefType, MediaType};
use crate::components::{Dropdown, DropdownLabel};

pub type FilterPredicate = fn(&Weak<RefCell<dyn ArticleData>>, inverted: &bool) -> bool;
const ALL_FILTERS: [Filter; 9] = [
	Filter::Media,
	Filter::Animated,
	Filter::NotMarkedAsRead,
	Filter::NotHidden,
	Filter::Liked,
	Filter::Reposted,
	Filter::PlainTweet,
	Filter::Repost { by_username: None },
	Filter::Quote { by_username: None },
];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Filter {
	Media,
	Animated,
	NotMarkedAsRead,
	NotHidden,
	Liked,
	Reposted,
	PlainTweet,
	Repost {
		by_username: Option<String>
	},
	Quote {
		by_username: Option<String>
	},
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
				Filter::Repost { .. } => "Not Repost",
				Filter::Quote { .. } => "No Quote",
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
				Filter::Repost { .. } => "Repost",
				Filter::Quote { .. } => "Has Quote",
			}
		}
	}
	pub fn iter() -> impl ExactSizeIterator<Item = &'static Filter> {
		ALL_FILTERS.iter()
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
			Filter::Repost { by_username } => {
				match article.referenced_article() {
					ArticleRefType::Repost(_) | ArticleRefType::QuoteRepost(_, _) => match by_username {
						Some(username) => &article.author_username() == username,
						None => true,
					},
					ArticleRefType::NoRef | ArticleRefType::Quote(_) => false,
				}
			}
			Filter::Quote { by_username } => {
				match article.referenced_article() {
					ArticleRefType::Quote(_) | ArticleRefType::QuoteRepost(_, _) => match by_username {
						Some(username) => &article.author_username() == username,
						None => true,
					},
					ArticleRefType::NoRef | ArticleRefType::Repost(_) => false,
				}
			}
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FilterInstance {
	pub filter: Filter,
	pub enabled: bool,
	pub inverted: bool,
}

impl FilterInstance {
	pub const fn new(filter: Filter) -> Self {
		Self {
			filter,
			enabled: true,
			inverted: false,
		}
	}

	pub const fn new_disabled(filter: Filter) -> Self {
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

pub const DEFAULT_FILTERS: [FilterInstance; 2] = [
	FilterInstance::new(Filter::NotMarkedAsRead),
	FilterInstance::new(Filter::NotHidden),
];

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterCollection(HashSet<FilterInstance>);

pub enum FilterMsg {
	ToggleFilterEnabled(FilterInstance),
	ToggleFilterInverted(FilterInstance),
	AddFilter((Filter, bool)),
	RemoveFilter(FilterInstance),
}

impl<const N: usize> From<[FilterInstance; N]> for FilterCollection {
	fn from(instances: [FilterInstance; N]) -> Self {
		Self(instances.into())
	}
}

impl Default for FilterCollection {
	fn default() -> Self {
		DEFAULT_FILTERS.into()
	}
}

impl Deref for FilterCollection {
	type Target = HashSet<FilterInstance>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for FilterCollection {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl IntoIterator for FilterCollection {
	type Item = FilterInstance;
	type IntoIter = std::collections::hash_set::IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

//Not sure if I'm supposed to redirect into_iter() to iter()...
impl<'a> IntoIterator for &'a FilterCollection {
	type Item = &'a FilterInstance;
	type IntoIter = std::collections::hash_set::Iter<'a, FilterInstance>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.iter()
	}
}

impl FilterCollection {
	pub fn new() -> Self {
		Self(HashSet::new())
	}

	pub fn update(&mut self, msg: FilterMsg) -> bool {
		match msg {
			FilterMsg::ToggleFilterEnabled(filter_instance) => {
				self.remove(&filter_instance);
				self.insert(FilterInstance {
					enabled: !filter_instance.enabled,
					..filter_instance
				});
				true
			}
			FilterMsg::ToggleFilterInverted(filter_instance) => {
				self.remove(&filter_instance);
				self.insert(FilterInstance {
					inverted: !filter_instance.inverted,
					..filter_instance
				});
				true
			}
			FilterMsg::AddFilter((filter, inverted)) => {
				self.insert(FilterInstance {filter, inverted, enabled: true});
				true
			}
			FilterMsg::RemoveFilter(filter_instance) => {
				self.remove(&filter_instance);
				true
			}
		}
	}
}

#[derive(Properties, PartialEq)]
pub struct FilterOptionsProps {
	pub filters: FilterCollection,
	pub callback: Callback<FilterMsg>,
}

#[function_component(FiltersOptions)]
pub fn filters_options(props: &FilterOptionsProps) -> Html {
	html! {
		<>
			{ for props.filters.iter().map(|filter_instance| {
				let toggle_enabled_callback = props.callback.clone();
				let toggle_enabled_filter_instance = filter_instance.clone();

				let toggle_inverted_callback = props.callback.clone();
				let toggle_inverted_filter_instance = filter_instance.clone();

				let remove_callback = props.callback.clone();
				let remove_filter_instance = filter_instance.clone();

				let (enabled_class, enabled_label) = match filter_instance.enabled {
					true => (Some("is-success"), "Enabled"),
					false => (None, "Disabled"),
				};
				let (inverted_class, inverted_label) = match filter_instance.inverted {
					true => (Some("is-info"), "Inverted"),
					false => (None, "Normal"),
				};

				html! {
					<div class="block field has-addons">
						<div class="field-label is-normal">
							<label class="label">{ filter_instance.filter.name(filter_instance.inverted) }</label>
						</div>
						<div class="field-body">
							<div class="control">
								<button
									class={classes!("button", enabled_class)}
									onclick={Callback::from(move |_| toggle_enabled_callback.emit(FilterMsg::ToggleFilterEnabled(toggle_enabled_filter_instance.clone())))}
								>
									{enabled_label}
								</button>
							</div>
							<div class="control">
								<button
									class={classes!("button", inverted_class)}
									onclick={Callback::from(move |_| toggle_inverted_callback.emit(FilterMsg::ToggleFilterInverted(toggle_inverted_filter_instance.clone())))}
								>
									{inverted_label}
								</button>
							</div>
							<div class="control">
								<button
									class="button"
									onclick={Callback::from(move |_| remove_callback.emit(FilterMsg::RemoveFilter(remove_filter_instance.clone())))}
								>
									{"Remove"}
								</button>
							</div>
						</div>
					</div>
				}
			}) }
			// TODO has-addons
			<Dropdown current_label={DropdownLabel::Text("New Filter".to_owned())}>
				{ for Filter::iter().cloned().map(|filter| {
					let callback = props.callback.clone();
					let filter_c = filter.clone();
					html! {
						<a class="dropdown-item" onclick={Callback::from(move |_| callback.emit(FilterMsg::AddFilter((filter_c.clone(), false))))}>
							{ filter.name(false) }
						</a>
					}
				}) }
			</Dropdown>
			<Dropdown current_label={DropdownLabel::Text("New Inverted Filter".to_owned())}>
				{ for Filter::iter().cloned().map(|filter| {
					let callback = props.callback.clone();
					let filter_c = filter.clone();
					html! {
						<a class="dropdown-item" onclick={Callback::from(move |_| callback.emit(FilterMsg::AddFilter((filter_c.clone(), true))))}>
							{ filter.name(true) }
						</a>
					}
				}) }
			</Dropdown>
		</>
	}
}