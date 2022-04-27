use std::rc::{Rc, Weak};
use std::cell::RefCell;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged, Dispatched, Dispatcher};
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlInputElement};
use rand::{seq::SliceRandom, thread_rng};

pub mod sort_methods;
pub mod agent;
pub mod filters;
pub mod timeline_container;
mod containers;
mod autoscroll;

pub use containers::Container;
use autoscroll::{AutoScroll, start_autoscroll, scroll_to_top};
use containers::{view_container, Props as ContainerProps, ContainerMsg};
use filters::{FilterCollection, FilterMsg, FiltersOptions};
use sort_methods::SortMethod;
use agent::{TimelineAgent, Request as TimelineAgentRequest};
use crate::articles::{ArticleView, ArticleRefType, ArticleWeak, ArticleBox, weak_actual_article};
use crate::services::endpoint_agent::{EndpointAgent, Request as EndpointRequest};
use crate::modals::ModalCard;
use crate::choose_endpoints::ChooseEndpoints;
use crate::components::{Dropdown, DropdownLabel, FA, IconSize};
use crate::services::article_actions::{ArticleActionsAgent, Request as ArticleActionsRequest, Response as ArticleActionsResponse};
use crate::services::storages::{hide_article, mark_article_as_read};
use crate::settings::{AppSettings, AppSettingsOverride, ArticleFilteredMode, view_on_media_click_setting, view_article_filtered_mode_setting, view_keep_column_count_setting, view_masonry_independent_columns_setting, ChangeSettingMsg};
use crate::{TimelineAgentResponse, TimelineEndpointWrapper};

pub type TimelineId = i8;

pub struct ArticleStruct {
	pub weak: ArticleWeak,
	//TODO Rename or describe
	pub included: bool,
	pub boxed: ArticleBox,
	boxed_actual_article_index_opt: Option<usize>,
	boxed_actual_article_opt: Option<ArticleBox>,
	pub boxed_refs: Vec<ArticleRefType<ArticleBox>>,
}

impl ArticleStruct {
	pub fn global_id(&self) -> String {
		format!("{}{}", &self.boxed.service(), &self.boxed.id())
	}

	pub fn boxed_actual_article(&self) -> &ArticleBox {
		match &self.boxed_actual_article_opt {
			Some(a) => a,
			None => &self.boxed,
		}
	}
}

impl Clone for ArticleStruct {
	fn clone(&self) -> Self {
		Self {
			weak: self.weak.clone(),
			included: self.included,
			boxed: self.boxed.clone_data(),
			boxed_actual_article_index_opt: self.boxed_actual_article_index_opt,
			boxed_actual_article_opt: self.boxed_actual_article_opt.as_ref().map(|a| a.clone_data()),
			boxed_refs: self.boxed_refs.iter().map(|ref_article| ref_article.clone_data()).collect(),
		}
	}
}

impl PartialEq for ArticleStruct {
	fn eq(&self, other: &Self) -> bool {
		Weak::ptr_eq(&self.weak, &other.weak) &&
			self.included == other.included &&
			&self.boxed == &other.boxed &&
			self.boxed_refs == other.boxed_refs
	}
}

pub struct Timeline {
	endpoints: Rc<RefCell<Vec<TimelineEndpointWrapper>>>,
	articles: Vec<ArticleWeak>,
	options_shown: bool,
	compact: bool,
	animated_as_gifs: bool,
	hide_text: bool,
	endpoint_agent: Dispatcher<EndpointAgent>,
	filters: FilterCollection,
	sort_method: (Option<SortMethod>, bool),
	_container: Container,
	_column_count: u8,
	width: u8,
	article_view: ArticleView,
	show_choose_endpoint: bool,
	container_ref: NodeRef,
	autoscroll: Rc<RefCell<AutoScroll>>,
	article_actions: Box<dyn Bridge<ArticleActionsAgent>>,
	timeline_agent: Box<dyn Bridge<TimelineAgent>>,
	use_section: bool,
	section: (usize, usize),
	rtl: bool,
	lazy_loading: bool,
	app_settings_override: AppSettingsOverride,
	should_organize_articles: bool,
}

pub enum Msg {
	Refresh,
	LoadBottom,
	LoadTop,
	Refreshed(Vec<ArticleWeak>),
	RefreshFail,
	NewArticles(Vec<ArticleWeak>),
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
	ScrollTop,
	FilterMsg(FilterMsg),
	SetSortMethod(Option<&'static SortMethod>),
	ToggleSortReversed,
	SortOnce(&'static SortMethod),
	ActionsCallback(ArticleActionsResponse),
	SetMainTimeline,
	RemoveTimeline,
	ToggleSection,
	UpdateSection(Option<usize>, Option<usize>),
	ToggleLazyLoading,
	Redraw,
	MarkAllAsRead,
	HideAll,
	ChangeSetting(ChangeSettingMsg),
	BalanceContainer,
	ContainerCallback(ContainerMsg),
	TimelineAgentResponse(TimelineAgentResponse),
}

#[derive(Properties, Clone)]
pub struct Props {
	pub name: String,
	pub id: TimelineId,
	//TODO Split TimelineProps into TimelineData and TimelineProps
	#[prop_or_default]
	pub app_settings: Option<AppSettings>,
	#[prop_or_default]
	pub hide: bool,
	#[prop_or_default]
	pub endpoints: Option<Vec<TimelineEndpointWrapper>>,
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
	pub articles: Vec<ArticleWeak>,
	#[prop_or_default]
	pub filters: Option<FilterCollection>,
	#[prop_or_default]
	pub sort_method: Option<(SortMethod, bool)>,
	#[prop_or_default]
	pub compact: bool,
	#[prop_or_default]
	pub animated_as_gifs: bool,
	#[prop_or_default]
	pub hide_text: bool,
	#[prop_or_default]
	pub rtl: bool,
	#[prop_or(false)]
	pub modal: bool,
}

impl PartialEq for Props {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name &&
			self.id == other.id &&
			self.hide == other.hide &&
			self.app_settings == other.app_settings &&
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
		let endpoints = match ctx.props().endpoints.clone() {
			Some(endpoints) => Rc::new(RefCell::new(endpoints)),
			None => Rc::new(RefCell::new(Vec::new()))
		};

		let mut endpoint_agent = EndpointAgent::dispatcher();
		endpoint_agent.send(EndpointRequest::InitTimeline(ctx.props().id.clone(), endpoints.clone(), ctx.link().callback(Msg::NewArticles)));

		let mut timeline_agent = TimelineAgent::bridge(ctx.link().callback(Msg::TimelineAgentResponse));
		timeline_agent.send(TimelineAgentRequest::RegisterTimeline(ctx.props().id));

		Self {
			endpoints,
			articles: ctx.props().articles.clone(),
			options_shown: false,
			compact: ctx.props().compact,
			animated_as_gifs: ctx.props().animated_as_gifs,
			hide_text: ctx.props().hide_text,
			endpoint_agent,
			filters: ctx.props().filters.as_ref().map(|f| f.clone()).unwrap_or_else(|| FilterCollection::default()),
			sort_method: match ctx.props().sort_method {
				Some((method, reversed)) => (Some(method), reversed),
				None => (None, true)
			},
			_container: ctx.props().container,
			_column_count: ctx.props().column_count,
			width: ctx.props().width,
			article_view: ctx.props().article_view,
			show_choose_endpoint: false,
			container_ref: NodeRef::default(),
			autoscroll: Rc::new(RefCell::new(AutoScroll::default())),
			article_actions: ArticleActionsAgent::bridge(ctx.link().callback(Msg::ActionsCallback)),
			timeline_agent,
			use_section: false,
			section: (0, 50),
			rtl: ctx.props().rtl,
			lazy_loading: true,
			app_settings_override: AppSettingsOverride::default(),
			should_organize_articles: false,
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
				if ctx.props().main_timeline {
					self.timeline_agent.send(TimelineAgentRequest::SetMainContainer(c))
				} else {
					self._container = c;
				}
				true
			}
			Msg::ChangeArticleView(c) => {
				self.article_view = c;
				true
			}
			Msg::ChangeColumnCount(new_column_count) => {
				if ctx.props().main_timeline {
					self.timeline_agent.send(TimelineAgentRequest::SetMainColumnCount(new_column_count))
				} else {
					self._column_count = new_column_count;
				}
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
				start_autoscroll(&mut self.autoscroll, self.container_ref.clone());

				false
			}
			Msg::ScrollTop => {
				scroll_to_top(self.container_ref.cast::<Element>().unwrap());

				false
			}
			Msg::FilterMsg(msg) => self.filters.update(msg),
			Msg::SetSortMethod(new_method) => {
				self.sort_method.0 = new_method.map(|method| *method);
				true
			}
			Msg::SortOnce(method) => {
				self.articles.sort_by(|a, b| {
					match self.sort_method.1 {
						false => method.compare(&a, &b),
						true => method.compare(&a, &b).reverse(),
					}
				});
				true
			}
			Msg::ToggleSortReversed => {
				self.sort_method.1 = !self.sort_method.1;
				true
			}
			Msg::ActionsCallback(response) => {
				match response {
					//Could filter articles for perfs
					ArticleActionsResponse::RedrawTimelines(_articles) => true
				}
			}
			Msg::SetMainTimeline => {
				self.timeline_agent.send(TimelineAgentRequest::SetMainTimeline(ctx.props().id));
				false
			}
			Msg::RemoveTimeline => {
				self.timeline_agent.send(TimelineAgentRequest::RemoveTimeline(ctx.props().id));
				self.endpoint_agent.send(EndpointRequest::RemoveTimeline(ctx.props().id));

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
			Msg::Redraw => true,
			Msg::MarkAllAsRead => {
				let articles = self.filtered_sectioned_articles(ctx, None);
				for article in &articles {
					let strong = weak_actual_article(&article).upgrade().unwrap();
					let mut borrow = strong.borrow_mut();

					let new_marked_as_read = !borrow.marked_as_read();
					borrow.set_marked_as_read(new_marked_as_read);

					mark_article_as_read(borrow.service(), borrow.id(), new_marked_as_read);
				}

				self.article_actions.send(ArticleActionsRequest::RedrawTimelines(articles));
				false
			}
			Msg::HideAll => {
				let articles = self.filtered_sectioned_articles(ctx, None);
				for article in &articles {
					let strong = weak_actual_article(&article).upgrade().unwrap();
					let mut borrow = strong.borrow_mut();

					let new_hidden = !borrow.hidden();
					borrow.set_hidden(new_hidden);

					hide_article(borrow.service(), borrow.id(), new_hidden);
				}

				self.article_actions.send(ArticleActionsRequest::RedrawTimelines(articles));
				false
			}
			Msg::ChangeSetting(change_msg) => {
				match change_msg {
					ChangeSettingMsg::OnMediaClick(on_media_click) => self.app_settings_override.on_media_click = Some(on_media_click),
					ChangeSettingMsg::ArticleFilteredMode(article_filtered_mode) => self.app_settings_override.article_filtered_mode = Some(article_filtered_mode),
					ChangeSettingMsg::KeepColumnCount(keep_column_count) => self.app_settings_override.keep_column_count = Some(keep_column_count),
					ChangeSettingMsg::MasonryIndependentColumns(masonry_independent_columns) => self.app_settings_override.masonry_independent_columns = Some(masonry_independent_columns),
				}
				true
			}
			Msg::BalanceContainer => {
				self.should_organize_articles = true;
				true
			}
			Msg::ContainerCallback(container_msg) => match container_msg {
				ContainerMsg::Organized => {
					self.should_organize_articles = false;
					false
				}
			}
			Msg::TimelineAgentResponse(response) => match response {
				TimelineAgentResponse::BatchAction(action, filters) => {
					self.article_actions.send(ArticleActionsRequest::Action(action, self.filtered_sectioned_articles(ctx, Some(filters))));
					false
				}
				_ => false,
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let articles = self.sectioned_articles(ctx, None);

		let articles: Vec<ArticleStruct> = articles.into_iter()
			.map(|(a, included)| {
				let strong = a.upgrade().expect("upgrading article");
				let borrow = strong.borrow();
				ArticleStruct {
					weak: a,
					included,
					boxed: borrow.clone_data(),
					boxed_actual_article_index_opt: borrow.actual_article_index(),
					boxed_actual_article_opt: borrow.actual_article().map(|a| a.upgrade().unwrap().borrow().clone_data()),
					boxed_refs: borrow.referenced_articles().into_iter().map(|ref_article| match ref_article {
						ArticleRefType::Reposted(a) => ArticleRefType::Reposted(
							a.upgrade().unwrap().borrow().clone_data()
						),
						ArticleRefType::Quote(a) => ArticleRefType::Quote(
							a.upgrade().unwrap().borrow().clone_data()
						),
						ArticleRefType::RepostedQuote(a, q) => ArticleRefType::RepostedQuote(
							a.upgrade().unwrap().borrow().clone_data(),
							q.upgrade().unwrap().borrow().clone_data(),
						),
						ArticleRefType::Reply(a) => ArticleRefType::Reply(
							a.upgrade().unwrap().borrow().clone_data()
						),
					}).collect(),
				}
			}).collect();

		let style = if self.width > 1 {
			Some(format!("width: {}px", (self.width as i32) * 500))
		} else {
			None
		};

		let article_count = articles.len() as u8;
		let column_count = if self.app_settings(ctx).keep_column_count {
			self.column_count(ctx)
		} else {
			std::cmp::min(self.column_count(ctx), std::cmp::max(1, article_count))
		};

		html! {
			<div class={classes!("timeline", if ctx.props().main_timeline { Some("mainTimeline") } else { None }, if ctx.props().hide { Some("is-hidden") } else { None })} {style}>
				<ModalCard enabled={self.show_choose_endpoint} modal_title="Choose Endpoints" close_modal_callback={ctx.link().callback(|_| Msg::SetChooseEndpointModal(false))}>
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
						{ if let Container::Masonry = self.container(ctx) {
							html! {
								<button onclick={ctx.link().callback(|_| Msg::BalanceContainer)} title="Organize Container">
									<FA icon="scale-balanced" size={IconSize::Large}/>
								</button>
							}
						} else {
							html! {}
						} }
						<button onclick={ctx.link().callback(|_| Msg::Shuffle)} title="Shuffle">
							<FA icon="random" size={IconSize::Large}/>
						</button>
						<button onclick={ctx.link().callback(|_| Msg::Autoscroll)} title="Autoscroll" class="timelineAutoscroll">
							<FA icon="scroll" size={IconSize::Large}/>
						</button>
						<button onclick={ctx.link().callback(|_| Msg::Refresh)} title="Refresh">
							<FA icon="sync-alt" size={IconSize::Large}/>
						</button>
						<button onclick={ctx.link().callback(|_| Msg::LoadBottom)} title="Load Bottom">
							<FA icon="arrow-down" size={IconSize::Large}/>
						</button>
						<button onclick={ctx.link().callback(|_| Msg::LoadTop)} title="Load Top">
							<FA icon="arrow-up" size={IconSize::Large}/>
						</button>
						<button onclick={ctx.link().callback(|_| Msg::ToggleOptions)} title="Expand options">
							<FA icon="ellipsis-v" size={IconSize::Large}/>
						</button>
					</div>
				</div>
				{ self.view_options(ctx) }
				{ view_container(&self.container(ctx), yew::props! {ContainerProps {
					container_ref: self.container_ref.clone(),
					compact: self.compact,
					animated_as_gifs: self.animated_as_gifs,
					hide_text: self.hide_text,
					column_count,
					rtl: self.rtl,
					should_organize_articles: self.should_organize_articles,
					callback: ctx.link().callback(|container_msg| Msg::ContainerCallback(container_msg)),
					lazy_loading: self.lazy_loading,
					article_view: self.article_view,
					articles,
					app_settings: self.app_settings(ctx)
				}}) }
			</div>
		}
	}
}

impl Timeline {
	fn container(&self, ctx: &Context<Self>) -> Container {
		if ctx.props().main_timeline {
			ctx.props().container
		} else {
			self._container
		}
	}

	fn column_count(&self, ctx: &Context<Self>) -> u8 {
		if ctx.props().main_timeline {
			ctx.props().column_count
		} else {
			self._column_count
		}
	}

	fn app_settings(&self, ctx: &Context<Self>) -> AppSettings {
		ctx.props().app_settings.unwrap().override_settings(&self.app_settings_override)
	}

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
				{ match self.container(ctx) {
					Container::Column => html! {},
					_ => html! {
						<>
							<div class="block control">
								<label class="label">{"Column Count"}</label>
								<input class="input" type="number" value={self.column_count(ctx).to_string()} min=1 oninput={on_column_count_input}/>
							</div>
							{ view_keep_column_count_setting(self.app_settings(ctx).keep_column_count, ctx.link().callback(Msg::ChangeSetting)) }
						</>
					},
				} }
				{ match self.container(ctx) {
					Container::Masonry => view_masonry_independent_columns_setting(self.app_settings(ctx).masonry_independent_columns, ctx.link().callback(Msg::ChangeSetting)),
					_ => html! {},
				} }
				{ match ctx.props().main_timeline {
					true => html! {},
					false => html! {
						<div class="block control">
							<label class="label">{"Timeline Width"}</label>
							<input class="input" type="number" value={self.width.to_string()} min=1 oninput={on_width_input}/>
						</div>
					}
				} }
				<div class="block control">
					<Dropdown current_label={DropdownLabel::Text(self.container(ctx).name().to_string())}>
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
				{ match self.article_view {
					ArticleView::Social => html! {
						<>
							<div class="control">
								<label class="checkbox">
									<input type="checkbox" checked={self.compact} onclick={ctx.link().callback(|_| Msg::ToggleCompact)}/>
									{ " Compact articles" }
								</label>
							</div>
							<div class="block control">
								<label class="checkbox">
									<input type="checkbox" checked={self.hide_text} onclick={ctx.link().callback(|_| Msg::ToggleHideText)}/>
									{ " Hide text" }
								</label>
							</div>
						</>
					},
					ArticleView::Gallery => html! {},
				} }
				{ view_on_media_click_setting(self.app_settings(ctx).on_media_click, ctx.link().callback(Msg::ChangeSetting)) }
				{ view_article_filtered_mode_setting(self.app_settings(ctx).article_filtered_mode, ctx.link().callback(Msg::ChangeSetting)) }
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
					<button class="button" onclick={ctx.link().callback(|_| Msg::SetChooseEndpointModal(true))}>{"Change Endpoints"}</button>
				</div>
				{
					//TODO Add modal to timeline list
					if !ctx.props().main_timeline && !ctx.props().modal {
						html! {
							<div class="block control">
								<button class="button" onclick={ctx.link().callback(|_| Msg::SetMainTimeline)}>{"Set as main timeline"}</button>
							</div>
						}
					}else {
						html! {}
					}
				}
				<div class="block control">
					<button class="button" onclick={ctx.link().callback(|_| Msg::MarkAllAsRead)}>{"Mark listed articles as read"}</button>
				</div>
				<div class="block control">
					<button class="button" onclick={ctx.link().callback(|_| Msg::HideAll)}>{"Hide listed articles"}</button>
				</div>
				<div class="block control">
					<button class="button" onclick={ctx.link().callback(|_| Msg::Redraw)}>{"Redraw timeline"}</button>
				</div>
				<div class="block control">
					<button class="button is-danger" onclick={ctx.link().callback(|_| Msg::RemoveTimeline)}>{"Remove timeline"}</button>
				</div>
			</div>
		}
	}

	fn view_filters_options(&self, ctx: &Context<Self>) -> Html {
		html! {
			<div class="box">
				<FiltersOptions
					filters={self.filters.clone()}
					callback={ctx.link().callback(Msg::FilterMsg)}
				/>
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
		} else {
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

	fn sectioned_articles(&self, ctx: &Context<Self>, extra_filters: Option<FilterCollection>) -> Vec<(ArticleWeak, bool)> {
		let mut articles: Vec<(ArticleWeak, bool)> = self.articles.iter().cloned().map(|a| (a, true)).collect();

		let mut filters = self.filters.clone();
		if let Some(extra_filters) = extra_filters {
			filters.extend(extra_filters);
		}

		for instance in filters {
			if instance.enabled {
				articles = articles.into_iter().map(|(a, included)| {
					let strong = a.upgrade();
					if let Some(strong) = strong {
						(a, included && instance.filter.filter(&strong.borrow()) != instance.inverted)
					} else {
						(a, false)
					}
				}).collect();
			}
		}

		if let ArticleFilteredMode::Hidden = self.app_settings(ctx).article_filtered_mode {
			articles = articles.into_iter().filter(|(_, included)| *included).collect();
		}

		if let Some(method) = self.sort_method.0 {
			articles.sort_by(|(a, _), (b, _)| {
				match self.sort_method.1 {
					false => method.compare(&a, &b),
					true => method.compare(&a, &b).reverse(),
				}
			});
		}

		if self.use_section {
			articles = articles.into_iter()
				.skip(self.section.0)
				.take(self.section.1)
				.collect();
		}

		articles
	}

	fn filtered_sectioned_articles(&self, ctx: &Context<Self>, extra_filters: Option<FilterCollection>) -> Vec<ArticleWeak> {
		self.sectioned_articles(ctx, extra_filters).into_iter().filter_map(|(a, included)| if included {
			Some(a)
		} else {
			None
		}).collect()
	}
}
