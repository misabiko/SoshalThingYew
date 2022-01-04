use std::rc::{Rc, Weak};
use std::cell::RefCell;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged, Dispatched, Dispatcher};
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlInputElement};
use rand::{seq::SliceRandom, thread_rng};
use wasm_bindgen::closure::Closure;

pub mod sort_methods;
pub mod agent;
pub mod filters;
mod containers;

pub use containers::Container;
use containers::{view_container, Props as ContainerProps};
use filters::{Filter, default_filters};
use sort_methods::{SortMethod, compare};
use agent::{TimelineAgent, Request as TimelineAgentRequest};
use crate::articles::{ArticleView, ArticleData};
use crate::services::endpoint_agent::{EndpointAgent, Request as EndpointRequest, TimelineEndpoints};
use crate::modals::ModalCard;
use crate::choose_endpoints::ChooseEndpoints;
use crate::dropdown::{Dropdown, DropdownLabel};
use crate::services::article_actions::{ArticleActionsAgent, Response as ArticleActionsResponse};

pub type TimelineId = i8;

#[derive(Clone)]
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
	endpoint_agent: Dispatcher<EndpointAgent>,
	filters: Vec<Filter>,
	sort_method: (Option<SortMethod>, bool),
	container: Container,
	column_count: u8,
	width: u8,
	article_view: ArticleView,
	show_choose_endpoint: bool,
	container_ref: NodeRef,
	autoscroll: Rc<RefCell<Autoscroll>>,
	_article_actions: Box<dyn Bridge<ArticleActionsAgent>>,
	timeline_agent: Dispatcher<TimelineAgent>,
	use_section: bool,
	section: (usize, usize),
	rtl: bool,
	lazy_loading: bool,
}

pub enum Msg {
	Refresh,
	LoadBottom,
	LoadTop,
	Refreshed(Vec<Weak<RefCell<dyn ArticleData>>>),
	RefreshFail,
	NewArticles(Vec<Weak<RefCell<dyn ArticleData>>>),
	ClearArticles,
	ToggleOptions,
	ToggleCompact,
	ToggleAnimatedAsGifs,
	ToggleHideText,
	ChangeContainer(Container),
	ChangeArticleView(ArticleView),
	ChangeColumnCount(u8),
	ChangeWidth(u8),
	Shuffle,
	SetChooseEndpointModal(bool),
	Autoscroll,
	ToggleFilterEnabled(usize),
	ToggleFilterInverted(usize),
	SetSortMethod(Option<&'static SortMethod>),
	ToggleSortReversed,
	SortOnce(&'static SortMethod),
	ScrollTop,
	ActionsCallback(ArticleActionsResponse),
	SetMainTimeline,
	RemoveTimeline,
	ToggleSection,
	UpdateSection(Option<usize>, Option<usize>),
	ToggleLazyLoading,
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
	#[prop_or(Container::Column)]
	pub container: Container,
	#[prop_or(ArticleView::Social)]
	pub article_view: ArticleView,
	#[prop_or(1)]
	pub width: u8,
	#[prop_or(1)]
	pub column_count: u8,
	#[prop_or_default]
	pub children: Children,
	#[prop_or_default]
	pub articles: Vec<Weak<RefCell<dyn ArticleData>>>,
	#[prop_or_default]
	pub filters: Option<Vec<Filter>>,
	#[prop_or(Some((SortMethod::Id, false)))]
	pub sort_method: Option<(SortMethod, bool)>,
	#[prop_or_default]
	pub compact: bool,
	#[prop_or_default]
	pub animated_as_gifs: bool,
	#[prop_or_default]
	pub hide_text: bool,
	#[prop_or_default]
	pub rtl: bool,
}

impl PartialEq for Props {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name &&
			self.id == other.id &&
			self.hide == other.hide &&
			self.endpoints == other.endpoints &&
			self.main_timeline == other.main_timeline &&
			self.container == other.container &&
			self.width == other.width &&
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

		let mut endpoint_agent = EndpointAgent::dispatcher();
		endpoint_agent.send(EndpointRequest::InitTimeline(ctx.props().id.clone(), endpoints.clone(), ctx.link().callback(Msg::NewArticles)));

		Self {
			endpoints,
			articles: ctx.props().articles.clone(),
			options_shown: false,
			compact: ctx.props().compact.clone(),
			animated_as_gifs: ctx.props().animated_as_gifs.clone(),
			hide_text: ctx.props().hide_text.clone(),
			endpoint_agent,
			filters: ctx.props().filters.as_ref().map(|f| f.clone()).unwrap_or_else(|| default_filters()),
			sort_method: match ctx.props().sort_method {
				Some((method, reversed)) => (Some(method), reversed),
				None => (None, false)
			},
			container: ctx.props().container.clone(),
			column_count: ctx.props().column_count.clone(),
			width: ctx.props().width.clone(),
			article_view: ctx.props().article_view.clone(),
			show_choose_endpoint: false,
			container_ref: NodeRef::default(),
			autoscroll: Rc::new(RefCell::new(Autoscroll {
				direction: ScrollDirection::Up,
				speed: 3.0,
				anim: None,
			})),
			_article_actions: ArticleActionsAgent::bridge(ctx.link().callback(Msg::ActionsCallback)),
			timeline_agent: TimelineAgent::dispatcher(),
			use_section: false,
			section: (0, 50),
			rtl: ctx.props().rtl.clone(),
			lazy_loading: true,
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::Refresh => {
				self.endpoint_agent.send(EndpointRequest::Refresh(Rc::downgrade(&self.endpoints)));
				false
			}
			Msg::LoadBottom => {
				self.endpoint_agent.send(EndpointRequest::LoadBottom(Rc::downgrade(&self.endpoints)));
				false
			}
			Msg::LoadTop => {
				self.endpoint_agent.send(EndpointRequest::LoadTop(Rc::downgrade(&self.endpoints)));
				false
			}
			Msg::Refreshed(articles) => {
				self.articles.extend(articles);
				true
			}
			Msg::RefreshFail => false,
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
			Msg::ChangeArticleView(c) => {
				self.article_view = c;
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
				self.sort_method = (None, false);
				true
			}
			Msg::SetChooseEndpointModal(value) => {
				self.show_choose_endpoint = value;
				true
			}
			Msg::Autoscroll => {
				let old_direction = self.autoscroll.borrow().direction.clone();
				self.autoscroll.borrow_mut().direction = match old_direction {
					ScrollDirection::Up => ScrollDirection::Down,
					ScrollDirection::Down => ScrollDirection::Up,
				};

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
			Msg::SetSortMethod(new_method) => {
				self.sort_method.0 = new_method.map(|method| *method);
				true
			}
			Msg::SortOnce(method) => {
				self.articles.sort_by(|a, b| {
					match self.sort_method.1 {
						false => compare(method, &a, &b),
						true => compare(method, &a, &b).reverse(),
					}
				});
				true
			}
			Msg::ToggleSortReversed => {
				self.sort_method.1 = !self.sort_method.1;
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
					//Could filter articles for perfs
					ArticleActionsResponse::RedrawTimelines(_articles) => true
				}
			}
			Msg::SetMainTimeline => {
				self.timeline_agent.send(TimelineAgentRequest::SetMainTimeline(ctx.props().id.clone()));
				false
			}
			Msg::RemoveTimeline => {
				self.timeline_agent.send(TimelineAgentRequest::RemoveTimeline(ctx.props().id.clone()));
				self.endpoint_agent.send(EndpointRequest::RemoveTimeline(ctx.props().id.clone()));

				false
			}
			Msg::ToggleSection => {
				self.use_section = !self.use_section;
				true
			}
			Msg::UpdateSection(min, max) => {
				self.section = (
					min.unwrap_or_else(|| self.section.0),
					max.unwrap_or_else(|| self.section.1)
				);
				true
			}
			Msg::ToggleLazyLoading => {
				self.lazy_loading = !self.lazy_loading;
				true
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
				articles = articles.into_iter().filter(|a| (filter.predicate)(a, &filter.inverted)).collect();
			}
		}

		if let Some(method) = self.sort_method.0 {
			articles.sort_by(|a, b| {
				match self.sort_method.1 {
					false => compare(&method, &a, &b),
					true => compare(&method, &a, &b).reverse(),
				}
			});
		}

		if self.use_section {
			articles = articles.into_iter()
				.skip(self.section.0)
				.take(self.section.1)
				.collect();
		}

		let articles: Vec<(Weak<RefCell<dyn ArticleData>>, Box<dyn ArticleData>)> = articles.into_iter()
			.map(|a| {
				let data = a.upgrade().expect("upgrading article")
					.borrow().clone_data();
				(a, data)
			}).collect();

		let style = if self.width > 1 {
			Some(format!("width: {}px", (self.width as i32) * 500))
		} else {
			None
		};
		html! {
			<div class={classes!("timeline", if ctx.props().main_timeline { Some("mainTimeline") } else { None })} {style}>
				<ModalCard enabled={self.show_choose_endpoint.clone()} modal_title="Choose Endpoints" close_modal_callback={ctx.link().callback(|_| Msg::SetChooseEndpointModal(false))}>
					<ChooseEndpoints
						timeline_endpoints={Rc::downgrade(&self.endpoints)}
					/>
				</ModalCard>

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
						<button onclick={ctx.link().callback(|_| Msg::LoadTop)} title="Load Top">
							<span class="icon">
								<i class="fas fa-arrow-up fa-lg"/>
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
					rtl: self.rtl,
					lazy_loading: self.lazy_loading,
					article_view: self.article_view.clone(),
					articles
				}}) }
			</div>
		}
	}
}

impl Timeline {
	//TODO Collapse boxes
	//TODO Change scrollbar color
	//TODO Move options to separate file/component?
	fn view_options(&self, ctx: &Context<Self>) -> Html {
		if self.options_shown {
			html! {
				<div class="timelineOptions">
					{ self.view_container_options(ctx) }
					{ self.view_section_options(ctx) }
					{ self.view_articles_options(ctx) }
					{ self.view_timeline_options(ctx) }
					{ self.view_filters_options(ctx) }
					{ self.view_sort_options(ctx) }
				</div>
			}
		} else {
			html! {}
		}
	}

	fn view_container_options(&self, ctx: &Context<Self>) -> Html {
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

		html! {
			<div class="box">
				{ match self.container {
					Container::Column => html! {},
					_ => html! {
						<div class="block control">
							<label class="label">{"Column Count"}</label>
							<input class="input" type="number" value={self.column_count.clone().to_string()} min=1 oninput={on_column_count_input}/>
						</div>
					},
				} }
				<div class="block control">
					<label class="label">{"Timeline Width"}</label>
					<input class="input" type="number" value={self.width.clone().to_string()} min=1 oninput={on_width_input}/>
				</div>
				<div class="block control">
					<Dropdown current_label={DropdownLabel::Text(self.container.name().to_string())}>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ChangeContainer(Container::Column))}> {"Column"} </a>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ChangeContainer(Container::Row))}> {"Row"} </a>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ChangeContainer(Container::Masonry))}> {"Masonry"} </a>
					</Dropdown>
				</div>
			</div>
		}
	}

	fn view_section_options(&self, ctx: &Context<Self>) -> Html {
		let on_min_input = ctx.link().batch_callback(|e: InputEvent|
			e.target()
				.and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
				.and_then(|i| i.value().parse::<usize>().ok())
				.map(|v| Msg::UpdateSection(Some(v), None))
		);
		let on_max_input = ctx.link().batch_callback(|e: InputEvent|
			e.target()
				.and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
				.and_then(|i| i.value().parse::<usize>().ok())
				.map(|v| Msg::UpdateSection(None, Some(v)))
		);

		html! {
			<div class="box">
				<div class="block control">
					<label class="label">{"Section"}</label>
					<label class="checkbox">
						<input type="checkbox" checked={self.use_section} onclick={ctx.link().callback(|_| Msg::ToggleSection)}/>
						{ " Limit listed articles" }
					</label>
				</div>
				{match self.use_section {
					true => html! {
						<>
							<div class="block control">
								<label class="label">{"Min"}</label>
								<input class="input" type="number" value={self.section.0.to_string()} min=0 max={self.section.1.to_string()} oninput={on_min_input}/>
							</div>
							<div class="block control">
								<label class="label">{"Max"}</label>
								<input class="input" type="number" value={self.section.1.to_string()} min={self.section.0.to_string()} oninput={on_max_input}/>
							</div>
						</>
					},
					false => html! {},
				}}
			</div>
		}
	}

	fn view_articles_options(&self, ctx: &Context<Self>) -> Html {
		html! {
			<div class="box">
				<div class="block control">
					<label class="label">{"Component"}</label>
					<Dropdown current_label={DropdownLabel::Text(self.article_view.name().to_string())}>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ChangeArticleView(ArticleView::Social))}> {"Social"} </a>
						<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ChangeArticleView(ArticleView::Gallery))}> {"Gallery"} </a>
					</Dropdown>
				</div>
				<div class="control">
					<label class="checkbox">
						<input type="checkbox" checked={self.compact} onclick={ctx.link().callback(|_| Msg::ToggleCompact)}/>
						{ " Compact articles" }
					</label>
				</div>
				<div class="control">
					<label class="checkbox">
						<input type="checkbox" checked={self.animated_as_gifs} onclick={ctx.link().callback(|_| Msg::ToggleAnimatedAsGifs)}/>
						{ " Show all animated as gifs" }
					</label>
				</div>
				<div class="control">
					<label class="checkbox">
						<input type="checkbox" checked={self.lazy_loading} onclick={ctx.link().callback(|_| Msg::ToggleLazyLoading)}/>
						{ " Lazy media loading" }
					</label>
				</div>
				<div class="block control">
					<label class="checkbox">
						<input type="checkbox" checked={self.hide_text} onclick={ctx.link().callback(|_| Msg::ToggleHideText)}/>
						{ " Hide text" }
					</label>
				</div>
				<div class="block control">
					<button class="button is-danger" onclick={ctx.link().callback(|_| Msg::ClearArticles)}>{"Clear Articles"}</button>
				</div>
			</div>
		}
	}

	fn view_timeline_options(&self, ctx: &Context<Self>) -> Html {
		html! {
			<div class="box">
				<div class="block control">
					<label class="label">{"Endpoint"}</label>
					<button class="button" onclick={ctx.link().callback(|_| Msg::SetChooseEndpointModal(true))}>{"Change"}</button>
				</div>
				{
					match ctx.props().main_timeline.clone() {
						false => html! {
							<div class="block control">
								<button class="button" onclick={ctx.link().callback(|_| Msg::SetMainTimeline)}>{"Set as main timeline"}</button>
							</div>
						},
						true => html! {}
					}
				}
				<div class="block control">
					<button class="button is-danger" onclick={ctx.link().callback(|_| Msg::RemoveTimeline)}>{"Remove timeline"}</button>
				</div>
			</div>
		}
	}

	fn view_filters_options(&self, ctx: &Context<Self>) -> Html {
		html! {
			<div class="box">
				{ for self.filters.iter().enumerate().map(|(i, filter)| {
					let enabled_i = i.clone();
					let (enabled_class, enabled_label) = match filter.enabled {
						true => (Some("is-success"), "Enabled"),
						false => (None, "Disabled"),
					};
					let (inverted_class, inverted_label) = match filter.inverted {
						true => (Some("is-info"), "Inverted"),
						false => (None, "Normal"),
					};

					html! {
						<div class="block field has-addons">
							<div class="field-label is-normal">
								<label class="label">{ filter.name.clone() }</label>
							</div>
							<div class="field-body">
								<div class="control">
									<button class={classes!("button", enabled_class)} onclick={ctx.link().callback(move |_| Msg::ToggleFilterEnabled(enabled_i))}>{enabled_label}</button>
								</div>
								<div class="control">
									<button class={classes!("button", inverted_class)} onclick={ctx.link().callback(move |_| Msg::ToggleFilterInverted(i))}>{inverted_label}</button>
								</div>
							</div>
						</div>
					}
				}) }
			</div>
		}
	}

	fn view_sort_options(&self, ctx: &Context<Self>) -> Html {
		let current_method_name = self.sort_method.0.map(|method| (method.to_string()));
		let sort_once = if self.sort_method.0.is_none() {
			html! {
				<div class="control">
					<Dropdown current_label={DropdownLabel::Text("Sort once".to_owned())}>
						{ for SortMethod::iter().map(|method| html! {
							<a class="dropdown-item" onclick={ctx.link().callback(move |_| Msg::SortOnce(method))}>
								{ format!("{} - {}", method, method.direction_label(self.sort_method.1)) }
							</a>
						}) }
					</Dropdown>
				</div>
			}
		}else {
			html! {}
		};

		html! {
			<div class="box">
				<div class="block field has-addons">
					<div class="field-label is-normal">
						<label class="label">{"Sort Method"}</label>
					</div>
					<div class="field-body">
						<div class="control">
							<Dropdown current_label={DropdownLabel::Text(current_method_name.unwrap_or("Unsorted".to_owned()))}>
								{ for SortMethod::iter().map(|method| html! {
									<a class="dropdown-item" onclick={ctx.link().callback(move |_| Msg::SetSortMethod(Some(method)))}>
										{ format!("{} - {}", method, method.direction_label(self.sort_method.1)) }
									</a>
								}) }
								<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::SetSortMethod(None))}> { "Unsorted" } </a>
							</Dropdown>
						</div>
						<div class="control">
							<button class="button" onclick={ctx.link().callback(|_| Msg::ToggleSortReversed)}>
								{ match self.sort_method.0 {
									Some(method) => method.direction_label(self.sort_method.1),
									None => if self.sort_method.1 { "Reversed" } else { "Normal" }
								} }
							</button>
						</div>
						{ sort_once }
					</div>
				</div>
			</div>
		}
	}
}
