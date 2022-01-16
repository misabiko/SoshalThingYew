use std::rc::Weak;
use std::cell::{Ref, RefCell};
use serde::{Serialize, Deserialize};
use yew::prelude::*;
use std::collections::HashSet;

use crate::articles::{ArticleData, ArticleMedia, ArticleRefType, MediaType};
use crate::components::{Dropdown, DropdownLabel};

pub type FilterPredicate = fn(&Weak<RefCell<dyn ArticleData>>, inverted: &bool) -> bool;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
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
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

#[derive(Properties, PartialEq)]
pub struct FilterOptionsProps {
	pub filters: HashSet<FilterInstance>,
	pub toggle_enabled_callback: Callback<FilterInstance>,
	pub toggle_inverted_callback: Callback<FilterInstance>,
	pub remove_callback: Callback<FilterInstance>,
	pub add_callback: Callback<(Filter, bool)>,
}

#[function_component(FiltersOptions)]
pub fn filters_options(props: &FilterOptionsProps) -> Html {
	html! {
		<>
			{ for props.filters.iter().map(|filter_instance| {
				let toggle_enabled_callback = props.toggle_enabled_callback.clone();
				let toggle_inverted_callback = props.toggle_inverted_callback.clone();
				let remove_callback = props.remove_callback.clone();

				let (enabled_class, enabled_label) = match filter_instance.enabled {
					true => (Some("is-success"), "Enabled"),
					false => (None, "Disabled"),
				};
				let (inverted_class, inverted_label) = match filter_instance.inverted {
					true => (Some("is-info"), "Inverted"),
					false => (None, "Normal"),
				};
				let filter_instance = *filter_instance;

				html! {
					<div class="block field has-addons">
						<div class="field-label is-normal">
							<label class="label">{ filter_instance.filter.name(filter_instance.inverted) }</label>
						</div>
						<div class="field-body">
							<div class="control">
								<button class={classes!("button", enabled_class)} onclick={Callback::from(move |_| toggle_enabled_callback.emit(filter_instance))}>{enabled_label}</button>
							</div>
							<div class="control">
								<button class={classes!("button", inverted_class)} onclick={Callback::from(move |_| toggle_inverted_callback.emit(filter_instance))}>{inverted_label}</button>
							</div>
							<div class="control">
								<button class="button" onclick={Callback::from(move |_| remove_callback.emit(filter_instance))}>{"Remove"}</button>
							</div>
						</div>
					</div>
				}
			}) }
			// TODO has-addons
			<Dropdown current_label={DropdownLabel::Text("New Filter".to_owned())}>
				{ for Filter::iter().map(|filter| {
					let add_callback = props.add_callback.clone();
					html! {
						<a class="dropdown-item" onclick={Callback::from(move |_| add_callback.emit((*filter, false)))}>
							{ filter.name(false) }
						</a>
					}
				}) }
			</Dropdown>
			<Dropdown current_label={DropdownLabel::Text("New Inverted Filter".to_owned())}>
				{ for Filter::iter().map(|filter| {
					let add_callback = props.add_callback.clone();
					html! {
						<a class="dropdown-item" onclick={Callback::from(move |_| add_callback.emit((*filter, true)))}>
							{ filter.name(true) }
						</a>
					}
				}) }
			</Dropdown>
		</>
	}
}