use std::cell::Ref;
use serde::{Serialize, Deserialize};
use yew::prelude::*;
use std::ops::{Deref, DerefMut};
use web_sys::HtmlInputElement;
use wasm_bindgen::JsCast;

use crate::articles::{ArticleData, ArticleMedia, ArticleRefType, ArticleWeak, MediaType};
use crate::components::{Dropdown, DropdownLabel};

pub type FilterPredicate = fn(&ArticleWeak, inverted: &bool) -> bool;

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
		by_username: Option<String>,
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
				Filter::PlainTweet => "Not a Plain Tweet",
				Filter::Repost { .. } => "Not a Repost",
				Filter::Quote { .. } => "No a Quote",
			}
		} else {
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

	pub fn iter() -> impl ExactSizeIterator<Item=&'static Filter> {
		ALL_FILTERS.iter()
	}

	//TODO Pass &ArticleBox?
	pub fn filter(&self, article: &Ref<dyn ArticleData>) -> bool {
		match self {
			Filter::Media => {
				article.referenced_articles().into_iter().any(|ref_article| match ref_article {
					ArticleRefType::Reposted(a) => a.upgrade().map(|r| !r.borrow().media().is_empty()).unwrap_or(false),
					ArticleRefType::Quote(a) => (a.upgrade().map(|r| !r.borrow().media().is_empty()).unwrap_or(false) || !article.media().is_empty()),
					ArticleRefType::RepostedQuote(a, q) => (q.upgrade().map(|r| !r.borrow().media().is_empty()).unwrap_or(false) || a.upgrade().map(|r| !r.borrow().media().is_empty()).unwrap_or(false) || !article.media().is_empty()),
				})
			}
			Filter::Animated => {
				article.referenced_articles().into_iter().any(|ref_article| match ref_article {
					ArticleRefType::Reposted(a) => a.upgrade().map(|r| r.borrow().media().iter().any(|m| is_animated(m))).unwrap_or(false),
					ArticleRefType::Quote(a) => (a.upgrade().map(|r| r.borrow().media().iter().any(|m| is_animated(m))).unwrap_or(false) || (article.media().iter().any(|m| is_animated(m)))),
					ArticleRefType::RepostedQuote(a, q) => (q.upgrade().map(|r| r.borrow().media().iter().any(|m| is_animated(m))).unwrap_or(false) || a.upgrade().map(|r| r.borrow().media().iter().any(|m| is_animated(m))).unwrap_or(false) || (article.media().iter().any(|m| is_animated(m)))),
				})
			}
			Filter::NotMarkedAsRead => {
				match article.actual_article() {
					Some(a) => !a.upgrade().unwrap().borrow().marked_as_read(),
					None => !article.marked_as_read(),
				}
			}
			Filter::NotHidden => {
				match article.actual_article() {
					Some(a) => !a.upgrade().unwrap().borrow().hidden(),
					None => !article.hidden(),
				}
			}
			Filter::Liked => {
				match article.actual_article() {
					Some(a) => !a.upgrade().unwrap().borrow().liked(),
					None => !article.liked(),
				}
			}
			Filter::Reposted => {
				match article.actual_article() {
					Some(a) => !a.upgrade().unwrap().borrow().reposted(),
					None => !article.reposted(),
				}
			}
			Filter::PlainTweet => {
				!article.referenced_articles().into_iter()
					.any(|a| matches!(a, ArticleRefType::Reposted(_) | ArticleRefType::Quote(_) | ArticleRefType::RepostedQuote(_, _)))
			}
			Filter::Repost { by_username } => {
				article.referenced_articles().into_iter().any(|a| match a {
					ArticleRefType::Reposted(_) | ArticleRefType::RepostedQuote(_, _) => match by_username {
						Some(username) => &article.author_username() == username,
						None => true,
					},
					ArticleRefType::Quote(_) => false,
				})
			}
			Filter::Quote { by_username } => {
				article.referenced_articles().into_iter().any(|a| match a {
					ArticleRefType::Quote(_) | ArticleRefType::RepostedQuote(_, _) => match by_username {
						Some(username) => &article.author_username() == username,
						None => true,
					},
					ArticleRefType::Reposted(_) => false,
				})
			}
		}
	}

	fn parameter_view(&self, callback: Callback<(u8, Event)>) -> Html {
		match self {
			Filter::Repost { by_username } | Filter::Quote { by_username } => {
				html! {
					<div class="field has-addons">
						<div class="field-label is-small">
							<label class="label">{ "Username" }</label>
						</div>
						<div class="field-body">
							<div class="control">
								<input type="text" class="input" onchange={move |input| callback.emit((0, input))} value={by_username.clone()}/>
							</div>
						</div>
					</div>
				}
			}
			_ => html! {}
		}
	}

	fn parameter_change(&mut self, param_index: u8, event: Event) -> bool {
		match self {
			Filter::Repost { by_username } | Filter::Quote { by_username } => {
				match param_index {
					0 => {
						let new_username = event.target().unwrap()
							.dyn_into::<HtmlInputElement>().unwrap()
							.value();
						match by_username {
							Some(username) => if new_username == *username {
								false
							} else {
								if new_username.is_empty() {
									*by_username = None;
								} else {
									*by_username = Some(new_username);
								}
								true
							},
							None => if new_username.is_empty() {
								false
							} else {
								*by_username = Some(new_username);
								true
							},
						}
					}
					_ => false
				}
			}
			_ => false,
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

pub enum FilterMsg {
	ToggleFilterEnabled(usize),
	ToggleFilterInverted(usize),
	AddFilter((Filter, bool)),
	RemoveFilter(usize),
	ParameterChange(usize, u8, Event),
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterCollection(Vec<FilterInstance>);

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
	type Target = Vec<FilterInstance>;

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
	type IntoIter = std::vec::IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

//Not sure if I'm supposed to redirect into_iter() to iter()...
impl<'a> IntoIterator for &'a FilterCollection {
	type Item = &'a FilterInstance;
	type IntoIter = core::slice::Iter<'a, FilterInstance>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.iter()
	}
}

impl FilterCollection {
	pub fn new() -> Self {
		Self(Vec::new())
	}

	pub fn update(&mut self, msg: FilterMsg) -> bool {
		match msg {
			FilterMsg::ToggleFilterEnabled(index) => {
				self[index].enabled = !self[index].enabled;
				true
			}
			FilterMsg::ToggleFilterInverted(index) => {
				self[index].inverted = !self[index].inverted;
				true
			}
			FilterMsg::AddFilter((filter, inverted)) => {
				self.push(FilterInstance { filter, inverted, enabled: true });
				true
			}
			FilterMsg::RemoveFilter(index) => {
				self.remove(index);
				true
			}
			FilterMsg::ParameterChange(index, param_index, event) => {
				self[index].filter.parameter_change(param_index, event)
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
			{ for props.filters.iter().enumerate().map(|(filter_index, filter_instance)| {
				let toggle_enabled_onclick = {
					let callback = props.callback.clone();
					Callback::from(move |_| callback.emit(FilterMsg::ToggleFilterEnabled(filter_index)))
				};

				let toggle_inverted_onclick = {
					let callback = props.callback.clone();
					Callback::from(move |_| callback.emit(FilterMsg::ToggleFilterInverted(filter_index)))
				};

				let remove_onclick = {
					let callback = props.callback.clone();
					Callback::from(move |_| callback.emit(FilterMsg::RemoveFilter(filter_index)))
				};

				let param_callback = {
					let callback = props.callback.clone();
					Callback::from(move |(param_index, event)| callback.emit(FilterMsg::ParameterChange(filter_index, param_index, event)))
				};

				let (enabled_class, enabled_label) = match filter_instance.enabled {
					true => (Some("is-success"), "Enabled"),
					false => (None, "Disabled"),
				};
				let (inverted_class, inverted_label) = match filter_instance.inverted {
					true => (Some("is-info"), "Inverted"),
					false => (None, "Normal"),
				};

				html! {
					<>
						<div class="field has-addons">
							<div class="field-label is-normal">
								<label class="label">{ filter_instance.filter.name(filter_instance.inverted) }</label>
							</div>
							<div class="field-body">
								<div class="control">
									<button class={classes!("button", enabled_class)} onclick={toggle_enabled_onclick}>
										{enabled_label}
									</button>
								</div>
								<div class="control">
									<button class={classes!("button", inverted_class)} onclick={toggle_inverted_onclick}>
										{inverted_label}
									</button>
								</div>
								<div class="control">
									<button class="button" onclick={remove_onclick}>
										{"Remove"}
									</button>
								</div>
							</div>
						</div>
						{filter_instance.filter.parameter_view(param_callback)}
					</>
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