use std::rc::{Rc, Weak};
use std::cell::RefCell;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged, Dispatched, Dispatcher};
use yew_agent::utils::store::{Bridgeable, ReadOnly, StoreWrapper};
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlInputElement};
use rand::{seq::SliceRandom, thread_rng};
use wasm_bindgen::closure::Closure;

pub mod sort_methods;
pub mod agent;
mod containers;
mod filters;

use containers::{Container, view_container, Props as ContainerProps};
use filters::{Filter, default_filters};
use sort_methods::{SortMethod, default_sort_methods};
use agent::{TimelineAgent, Request as TimelineAgentRequest};
use crate::articles::{ArticleComponent, ArticleData, actual_article};
use crate::services::endpoints::{EndpointStore, Request as EndpointRequest, TimelineEndpoints};
use crate::modals::Modal;
use crate::choose_endpoints::ChooseEndpoints;
use crate::dropdown::{Dropdown, DropdownLabel};
use crate::services::article_actions::{ArticleActionsAgent, Response as ArticleActionsResponse};

pub type TimelineId = i8;

enum ScrollDirection {
	Up,
	Down,
}

struct AutoscrollAnim {
	request_id: i32,
	scroll_step: Closure<dyn FnMut()>,
	scroll_stop: Closure<dyn FnMut()>,
}

struct Autoscroll {
	direction: ScrollDirection,
	speed: f64,
	anim: Option<AutoscrollAnim>,
}

pub struct Timeline {
	endpoints: Rc<RefCell<TimelineEndpoints>>,
	articles: Vec<Weak<RefCell<dyn ArticleData>>>,
	options_shown: bool,
	compact: bool,
	animated_as_gifs: bool,
	hide_text: bool,
	endpoint_store: Box<dyn Bridge<StoreWrapper<EndpointStore>>>,
	filters: Vec<Filter>,
	sort_methods: Vec<SortMethod>,
	sort_method: Option<usize>,
	container: Container,
	show_container_dropdown: bool,
	show_article_component_dropdown: bool,
	column_count: u8,
	width: u8,
	article_component: ArticleComponent,
	show_choose_endpoint: bool,
	container_ref: NodeRef,
	autoscroll: Rc<RefCell<Autoscroll>>,
	_article_actions: Box<dyn Bridge<ArticleActionsAgent>>,
	timeline_agent: Dispatcher<TimelineAgent>,
}

pub enum Msg {
	Refresh,
	LoadBottom,
	Refreshed(Vec<Weak<RefCell<dyn ArticleData>>>),
	RefreshFail,
	NewArticles(Vec<Weak<RefCell<dyn ArticleData>>>),
	ClearArticles,
	EndpointStoreResponse(ReadOnly<EndpointStore>),
	ToggleOptions,
	ToggleCompact,
	ToggleAnimatedAsGifs,
	ToggleHideText,
	ChangeContainer(Container),
	ToggleContainerDropdown,
	ChangeArticleComponent(ArticleComponent),
	ToggleArticleComponentDropdown,
	ChangeColumnCount(u8),
	ChangeWidth(u8),
	Shuffle,
	SetChooseEndpointModal(bool),
	Autoscroll,
	ToggleFilterEnabled(usize),
	ToggleFilterInverted(usize),
	SetSortMethod(Option<usize>),
	ToggleSortReversed,
	ScrollTop,
	ActionsCallback(ArticleActionsResponse),
	SetMainTimeline,
	RemoveTimeline,
}

#[derive(Properties, Clone)]
pub struct Props {
	pub name: String,
	pub id: TimelineId,
	#[prop_or_default]
	pub hide: bool,
	#[prop_or_default]
	pub endpoints: Option<TimelineEndpoints>,
	#[prop_or_default]
	pub main_timeline: bool,
	#[prop_or(1)]
	pub column_count: u8,
	#[prop_or_default]
	pub children: Children,
	#[prop_or_default]
	pub articles: Vec<Weak<RefCell<dyn ArticleData>>>,
}

impl PartialEq for Props {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name &&
			self.id == other.id &&
			self.hide == other.hide &&
			self.endpoints == other.endpoints &&
			self.main_timeline == other.main_timeline &&
			self.column_count == other.column_count &&
			self.children == other.children &&
			self.articles.iter().zip(other.articles.iter())
				.all(|(ai, bi)| Weak::ptr_eq(&ai, &bi))
	}
}

impl Component for Timeline {
	type Message = Msg;
	type Properties = Props;

	fn create(ctx: &Context<Self>) -> Self {
		log::debug!("Creating timeline!");
		let endpoints = match ctx.props().endpoints.clone() {
			Some(endpoints) => Rc::new(RefCell::new(endpoints)),
			None => Rc::new(RefCell::new(TimelineEndpoints::default()))
		};

		let mut endpoint_store = EndpointStore::bridge(ctx.link().callback(Msg::EndpointStoreResponse));
		endpoint_store.send(EndpointRequest::InitTimeline(ctx.props().id.clone(), endpoints.clone(), ctx.link().callback(Msg::NewArticles)));

		Self {
			endpoints,
			articles: ctx.props().articles.clone(),
			options_shown: false,
			compact: false,
			animated_as_gifs: false,
			hide_text: false,
			endpoint_store,
			filters: default_filters(),
			sort_methods: default_sort_methods(),
			sort_method: Some(0),
			container: if ctx.props().main_timeline { Container::Masonry } else { Container::Column },
			show_container_dropdown: false,
			show_article_component_dropdown: false,
			column_count: ctx.props().column_count.clone(),
			width: 1,
			article_component: ArticleComponent::Social,
			show_choose_endpoint: false,
			container_ref: NodeRef::default(),
			autoscroll: Rc::new(RefCell::new(Autoscroll {
				direction: ScrollDirection::Down,
				speed: 3.0,
				anim: None,
			})),
			_article_actions: ArticleActionsAgent::bridge(ctx.link().callback(Msg::ActionsCallback)),
			timeline_agent: TimelineAgent::dispatcher(),
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::Refresh => {
				self.endpoint_store.send(EndpointRequest::Refresh(Rc::downgrade(&self.endpoints)));
				false
			}
			Msg::LoadBottom => {
				self.endpoint_store.send(EndpointRequest::LoadBottom(Rc::downgrade(&self.endpoints)));
				false
			}
			Msg::Refreshed(articles) => {
				self.articles.extend(articles);
				true
			}
			Msg::RefreshFail => false,
			Msg::EndpointStoreResponse(_) => false,
			Msg::NewArticles(articles) => {
				for a in articles {
					let exists = self.articles.iter()
						.any(
							|existing| existing.upgrade()
								.zip(a.upgrade())
								.map(|(e_s, a_s)| e_s.borrow().id() == a_s.borrow().id())
								.unwrap_or(false)
						);

					if !exists {
						self.articles.push(a);
					}
				}
				true
			}
			Msg::ClearArticles => {
				self.articles.clear();
				true
			}
			Msg::ToggleOptions => {
				self.options_shown = !self.options_shown;
				true
			}
			Msg::ToggleCompact => {
				self.compact = !self.compact;
				true
			}
			Msg::ToggleAnimatedAsGifs => {
				self.animated_as_gifs = !self.animated_as_gifs;
				true
			}
			Msg::ToggleHideText => {
				self.hide_text = !self.hide_text;
				true
			}
			Msg::ChangeContainer(c) => {
				self.container = c;
				true
			}
			Msg::ToggleContainerDropdown => {
				self.show_container_dropdown = !self.show_container_dropdown;
				true
			}
			Msg::ChangeArticleComponent(c) => {
				self.article_component = c;
				true
			}
			Msg::ToggleArticleComponentDropdown => {
				self.show_article_component_dropdown = !self.show_article_component_dropdown;
				true
			}
			Msg::ChangeColumnCount(new_column_count) => {
				self.column_count = new_column_count;
				true
			}
			Msg::ChangeWidth(new_width) => {
				self.width = new_width;
				true
			}
			Msg::Shuffle => {
				self.articles.shuffle(&mut thread_rng());
				self.sort_method = None;
				true
			}
			Msg::SetChooseEndpointModal(value) => {
				self.show_choose_endpoint = value;
				true
			}
			Msg::Autoscroll => {
				let anim_autoscroll = self.autoscroll.clone();
				let event_autoscroll = self.autoscroll.clone();
				let container_ref_c = self.container_ref.clone();

				let mut outer_borrow_mut = self.autoscroll.borrow_mut();

				let window = web_sys::window().expect("no global window");
				outer_borrow_mut.anim = {
					let anim = AutoscrollAnim {
						scroll_step: Closure::wrap(Box::new(move || {
							let mut borrow = anim_autoscroll.borrow_mut();
							if let Some(container) = container_ref_c.cast::<Element>() {
								let should_keep_scrolling = match borrow.direction {
									ScrollDirection::Up => container.scroll_top() > 0,
									ScrollDirection::Down => container.scroll_top() < container.scroll_height() - container.client_height(),
								};

								if should_keep_scrolling {
									container.scroll_by_with_x_and_y(0.0, match borrow.direction {
										ScrollDirection::Up => -borrow.speed,
										ScrollDirection::Down => borrow.speed,
									});
								} else {
									borrow.direction = match borrow.direction {
										ScrollDirection::Up => ScrollDirection::Down,
										ScrollDirection::Down => ScrollDirection::Up,
									};
								}
							}

							let mut anim = borrow.anim.as_mut().unwrap();
							anim.request_id = web_sys::window().expect("no global window")
								.request_animation_frame(anim.scroll_step.as_ref().unchecked_ref())
								.unwrap();
						}) as Box<dyn FnMut()>),
						request_id: 0,
						scroll_stop: Closure::once(Box::new(move || {
							let mut borrow = event_autoscroll.borrow_mut();
							if let Some(anim) = &borrow.anim {
								web_sys::window().expect("no global window")
									.cancel_animation_frame(anim.request_id)
									.unwrap();
							}

							borrow.anim = None;
						}) as Box<dyn FnOnce()>)
					};
					let mut options = web_sys::AddEventListenerOptions::new();
					window.add_event_listener_with_callback_and_add_event_listener_options(
						"mousedown",
						anim.scroll_stop.as_ref().unchecked_ref(),
						options.once(true),
					).unwrap();

					window.request_animation_frame(anim.scroll_step.as_ref().unchecked_ref()).unwrap();
					Some(anim)
				};


				false
			}
			Msg::ToggleFilterEnabled(filter_index) => {
				let mut filter = self.filters.get_mut(filter_index).unwrap();
				filter.enabled = !filter.enabled;
				true
			}
			Msg::ToggleFilterInverted(filter_index) => {
				let mut filter = self.filters.get_mut(filter_index).unwrap();
				filter.inverted = !filter.inverted;
				true
			}
			Msg::SetSortMethod(sort_index) => {
				self.sort_method = sort_index;
				true
			}
			Msg::ToggleSortReversed => {
				if let Some(sort_method) = self.sort_method {
					let mut sort_method = self.sort_methods.get_mut(sort_method.clone()).unwrap();
					sort_method.reversed = !sort_method.reversed;
				}
				true
			}
			Msg::ScrollTop => {
				if let Some(container) = self.container_ref.cast::<Element>() {
					let mut options = web_sys::ScrollToOptions::new();
					options.top(0.0);
					options.behavior(web_sys::ScrollBehavior::Smooth);
					container.scroll_to_with_scroll_to_options(&options);
				}
				false
			}
			Msg::ActionsCallback(response) => {
				match response {
					ArticleActionsResponse::Callback(_articles) => true
				}
			}
			Msg::SetMainTimeline => {
				self.timeline_agent.send(TimelineAgentRequest::SetMainTimeline(ctx.props().id.clone()));
				false
			}
			Msg::RemoveTimeline => {
				self.timeline_agent.send(TimelineAgentRequest::RemoveTimeline(ctx.props().id.clone()));
				self.endpoint_store.send(EndpointRequest::RemoveTimeline(ctx.props().id.clone()));

				false
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		if ctx.props().hide {
			return html! {}
		}

		let mut articles = self.articles.clone();
		for filter in &self.filters {
			if filter.enabled {
				articles = articles.into_iter().filter(|a| (filter.predicate)(a, filter.inverted.clone())).collect();
			}
		}

		if !self.sort_methods.is_empty() {
			if let Some(sort_method) = self.sort_method {
				let method = &self.sort_methods[sort_method.clone()];
				articles.sort_by(|a, b| {
					let (actual_a, actual_b) = (actual_article(&a), actual_article(&b));
					match method.reversed {
						false => (method.compare)(&actual_a, &actual_b),
						true => (method.compare)(&actual_a, &actual_b).reverse(),
					}
				});
			}
		}

		let style = if self.width > 1 {
			Some(format!("width: {}px", (self.width as i32) * 500))
		} else {
			None
		};
		html! {
			<div class={classes!("timeline", if ctx.props().main_timeline { Some("mainTimeline") } else { None })} {style}>
				<Modal enabled={self.show_choose_endpoint.clone()} modal_title="Choose Endpoints" close_modal_callback={ctx.link().callback(|_| Msg::SetChooseEndpointModal(false))}>
					<ChooseEndpoints
						timeline_endpoints={Rc::downgrade(&self.endpoints)}
					/>
				</Modal>

				<div class="timelineHeader">
					<div class="timelineLeftHeader">
						<strong onclick={ctx.link().callback(|_| Msg::ScrollTop)}>{ctx.props().name.clone()}</strong>
						{ if ctx.props().children.is_empty() {
							html! {}
						}else {
							html! {
								<div class="timelineButtons">
									{ for ctx.props().children.iter() }
								</div>
							}
						} }
					</div>
					<div class="timelineButtons">
						<button onclick={ctx.link().callback(|_| Msg::Shuffle)} title="Shuffle">
							<span class="icon">
								<i class="fas fa-random fa-lg"/>
							</span>
						</button>
						<button onclick={ctx.link().callback(|_| Msg::Autoscroll)} title="Autoscroll">
							<span class="icon">
								<i class="fas fa-scroll fa-lg"/>
							</span>
						</button>
						<button onclick={ctx.link().callback(|_| Msg::Refresh)} title="Refresh">
							<span class="icon">
								<i class="fas fa-sync-alt fa-lg"/>
							</span>
						</button>
						<button onclick={ctx.link().callback(|_| Msg::LoadBottom)} title="Load Bottom">
							<span class="icon">
								<i class="fas fa-arrow-down fa-lg"/>
							</span>
						</button>
						<button onclick={ctx.link().callback(|_| Msg::ToggleOptions)} title="Expand options">
							<span class="icon">
								<i class="fas fa-ellipsis-v fa-lg"/>
							</span>
						</button>
					</div>
				</div>
				{ self.view_options(ctx) }
				{ view_container(&self.container, yew::props! {ContainerProps {
					container_ref: self.container_ref.clone(),
					compact: self.compact,
					animated_as_gifs: self.animated_as_gifs,
					hide_text: self.hide_text,
					column_count: self.column_count,
					article_component: self.article_component.clone(),
					articles
				}}) }
			</div>
		}
	}
}

impl Timeline {
	fn view_options(&self, ctx: &Context<Self>) -> Html {
		let on_column_count_input = ctx.link().batch_callback(|e: InputEvent|
			e.target()
				.and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
				.and_then(|i| i.value().parse::<u8>().ok())
				.map(|v| Msg::ChangeColumnCount(v))
		);
		let on_width_input = ctx.link().batch_callback(|e: InputEvent|
			e.target()
				.and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
				.and_then(|i| i.value().parse::<u8>().ok())
				.map(|v| Msg::ChangeWidth(v))
		);
		if self.options_shown {
			html! {
				<div class="timelineOptions">
					<div class="block">
						<div class="control">
							<label class="label">{"Column Count"}</label>
							<input type="number" value={self.column_count.clone().to_string()} min=1 oninput={on_column_count_input}/>
						</div>
						<div class="control">
							<label class="label">{"Timeline Width"}</label>
							<input type="number" value={self.width.clone().to_string()} min=1 oninput={on_width_input}/>
						</div>
						<div class="control">
							<Dropdown current_label={DropdownLabel::Text(self.container.name().to_string())}>
								<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ChangeContainer(Container::Column))}> {"Column"} </a>
								<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ChangeContainer(Container::Row))}> {"Row"} </a>
								<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ChangeContainer(Container::Masonry))}> {"Masonry"} </a>
							</Dropdown>
						</div>
					</div>
					<div class="block">
						<div class="control">
							<Dropdown current_label={DropdownLabel::Text(self.article_component.name().to_string())}>
								<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ChangeArticleComponent(ArticleComponent::Social))}> {"Social"} </a>
								<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ChangeArticleComponent(ArticleComponent::Gallery))}> {"Gallery"} </a>
							</Dropdown>
						</div>
					</div>
					<div class="block">
						<div class="control">
							<label class="checkbox">
								<input type="checkbox" checked={self.compact} onclick={ctx.link().callback(|_| Msg::ToggleCompact)}/>
								{ "Compact articles" }
							</label>
						</div>
						<div class="control">
							<label class="checkbox">
								<input type="checkbox" checked={self.animated_as_gifs} onclick={ctx.link().callback(|_| Msg::ToggleAnimatedAsGifs)}/>
								{ "Show all animated as gifs" }
							</label>
						</div>
						<div class="control">
							<label class="checkbox">
								<input type="checkbox" checked={self.hide_text} onclick={ctx.link().callback(|_| Msg::ToggleHideText)}/>
								{ "Hide text" }
							</label>
						</div>
					</div>
					<div class="block">
						<div class="control">
							<label class="label">{"Endpoint"}</label>
							<button class="button" onclick={ctx.link().callback(|_| Msg::SetChooseEndpointModal(true))}>{"Change"}</button>
						</div>
					</div>
					<div class="block">
						<div class="control">
							<button class="button" onclick={ctx.link().callback(|_| Msg::ClearArticles)}>{"Clear Articles"}</button>
						</div>
					</div>
					{
						match ctx.props().main_timeline.clone() {
							false => html! {
								<div class="block">
									<div class="control">
										<button class="button" onclick={ctx.link().callback(|_| Msg::SetMainTimeline)}>{"Set as main timeline"}</button>
									</div>
								</div>
							},
							true => html! {}
						}
					}
					<div class="block">
						<div class="control">
							<button class="button" onclick={ctx.link().callback(|_| Msg::RemoveTimeline)}>{"Remove timeline"}</button>
						</div>
					</div>
					{ self.view_filters(ctx) }
					{ self.view_sort_method(ctx) }
				</div>
			}
		} else {
			html! {}
		}
	}

	fn view_filters(&self, ctx: &Context<Self>) -> Html {
		html! {
			<div class="block">
				{ for self.filters.iter().enumerate().map(|(i, filter)| {
					let enabled_i = i.clone();
					html! {
						<div class="control">
							<label class="label">{ filter.name.clone() }</label>
							<button class="button" onclick={ctx.link().callback(move |_| Msg::ToggleFilterEnabled(enabled_i))}>{if filter.enabled { "Enabled" } else { "Disabled" }}</button>
							<button class="button" onclick={ctx.link().callback(move |_| Msg::ToggleFilterInverted(i))}>{if filter.inverted { "Inverted" } else { "Normal" }}</button>
						</div>
					}
				}) }
			</div>
		}
	}

	fn view_sort_method(&self, ctx: &Context<Self>) -> Html {
		let current_method = if !self.sort_methods.is_empty() {
			if let Some(sort_method) = self.sort_method {
				let method = &self.sort_methods[sort_method.clone()];
				Some((method.name.clone(), method.reversed.clone()))
			} else {
				None
			}
		}else {
			None
		};

		html! {
			<div class="block">
				<div class="control">
					<Dropdown current_label={DropdownLabel::Text(current_method.as_ref().map(|m| m.0.clone()).unwrap_or("Unsorted".to_owned()))}>
						{ for self.sort_methods.iter().enumerate().map(|(i, method)| html! {
							<a class="dropdown-item" onclick={ctx.link().callback(move |_| Msg::SetSortMethod(Some(i)))}> {method.name.clone()} </a>
						})}
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::SetSortMethod(None))}> {"Unsorted"} </a>
					</Dropdown>
					<button class="button" onclick={ctx.link().callback(|_| Msg::ToggleSortReversed)}>{ if current_method.map(|m| m.1).unwrap_or(false) { "Reversed" } else { "Normal" }}</button>
				</div>
			</div>
		}
	}
}
