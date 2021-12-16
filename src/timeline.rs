use std::rc::Rc;
use yew::prelude::*;
use yew_agent::Bridge;
use yew_agent::utils::store::{StoreWrapper, ReadOnly, Bridgeable};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use rand::{thread_rng, seq::SliceRandom};
use std::cell::RefCell;

use crate::articles::{ArticleComponent, ArticleData, sort_by_id};
use crate::services::endpoints::{EndpointStore, TimelineEndpoints, StoreRequest as EndpointRequest};
use crate::containers::{Container, view_container, Props as ContainerProps};
use crate::modals::ChooseTimelineEndpointModal;

pub struct Timeline {
	endpoints: Rc<RefCell<TimelineEndpoints>>,
	articles: Vec<Rc<dyn ArticleData>>,	//TODO Use rc::Weak
	options_shown: bool,
	compact: bool,
	endpoint_store: Box<dyn Bridge<StoreWrapper<EndpointStore>>>,
	filters: Vec<fn(&Rc<dyn ArticleData>) -> bool>,
	container: Container,
	show_container_dropdown: bool,
	show_article_component_dropdown: bool,
	column_count: u8,
	width: u8,
	sorted: bool,
	article_component: ArticleComponent,
	show_choose_endpoint: bool,
}

pub enum Msg {
	Refresh,
	LoadBottom,
	Refreshed(Vec<Rc<dyn ArticleData>>),
	RefreshFail,
	NewArticles(Vec<Rc<dyn ArticleData>>),
	EndpointStoreResponse(ReadOnly<EndpointStore>),
	ToggleOptions,
	ToggleCompact,
	ChangeContainer(Container),
	ToggleContainerDropdown,
	ChangeArticleComponent(ArticleComponent),
	ToggleArticleComponentDropdown,
	ChangeColumnCount(u8),
	ChangeWidth(u8),
	Shuffle,
	SetChooseEndpointModal(bool),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
	pub name: String,
	#[prop_or_default]
	pub articles: Vec<Rc<dyn ArticleData>>,
	#[prop_or_default]
	pub endpoints: Option<TimelineEndpoints>,
	#[prop_or_default]
	pub main_timeline: bool,
	#[prop_or(1)]
	pub column_count: u8,
	#[prop_or_default]
	pub children: Children,
}

impl Component for Timeline {
	type Message = Msg;
	type Properties = Props;

	fn create(ctx: &Context<Self>) -> Self {
		let endpoints = match ctx.props().endpoints.clone() {
			Some(endpoints) => Rc::new(RefCell::new(endpoints)),
			None => Rc::new(RefCell::new(TimelineEndpoints {
				start: Vec::new(),
				refresh: Vec::new(),
			}))
		};

		let mut endpoint_store = EndpointStore::bridge(ctx.link().callback(Msg::EndpointStoreResponse));
		endpoint_store.send(EndpointRequest::InitTimeline(endpoints.clone(), ctx.link().callback(Msg::NewArticles)));

		Self {
			endpoints,
			articles: ctx.props().articles.clone(),
			options_shown: false,
			compact: false,
			endpoint_store,
			filters: vec![|a| a.media().len() > 0],
			container: if ctx.props().main_timeline { Container::Masonry } else { Container::Column },
			show_container_dropdown: false,
			show_article_component_dropdown: false,
			column_count: ctx.props().column_count.clone(),
			width: 1,
			sorted: true,
			article_component: ArticleComponent::Social,
			show_choose_endpoint: false,
		}
	}

	fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
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
					if !self.articles.iter().any(|existing| existing.id() == a.id()) {
						self.articles.push(a);
					}
				}
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
				self.sorted = false;
				true
			}
			Msg::SetChooseEndpointModal(value) => {
				self.show_choose_endpoint = value;
				true
			},
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let mut articles = self.articles.clone();
		for filter in &self.filters {
			articles = articles.into_iter().filter(filter).collect();
		}

		if self.sorted {
			articles.sort_by(sort_by_id);
		}

		let style = if self.width > 1 {
			Some(format!("width: {}px", (self.width as i32) * 500))
		} else {
			None
		};
		html! {
			<div class={classes!("timeline", if ctx.props().main_timeline { Some("mainTimeline") } else { None })} {style}>
				{
					match self.show_choose_endpoint {
						true => html! {<ChooseTimelineEndpointModal
							timeline_endpoints={Rc::downgrade(&self.endpoints)}
							close_modal_callback={ctx.link().callback(|_| Msg::SetChooseEndpointModal(false))}
						/>},
						false => html! {},
					}
				}
				<div class="timelineHeader">
					<div class="timelineLeftHeader">
						<strong>{&ctx.props().name}</strong>
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
					compact: self.compact,
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
					<div class="control">
						<label class="label">{"Column Count"}</label>
						<input type="number" value={self.column_count.clone().to_string()} min=1 oninput={on_column_count_input}/>
					</div>
					<div class="control">
						<label class="label">{"Timeline Width"}</label>
						<input type="number" value={self.width.clone().to_string()} min=1 oninput={on_width_input}/>
					</div>
					<div class="control">
						<div class={classes!("dropdown", if self.show_container_dropdown { Some("is-active") } else { None })}>
							<div class="dropdown-trigger">
								<button class="button" onclick={ctx.link().callback(|_| Msg::ToggleContainerDropdown)}>
									<span>{self.container.name()}</span>
									<span class="icon is-small">
										<i class="fas fa-angle-down"/>
									</span>
								</button>
							</div>
							<div class="dropdown-menu">
								<div class="dropdown-content">
									<button class="dropdown-item"
										onclick={ctx.link().callback(|_| Msg::ChangeContainer(Container::Column))}
									> {"Column"} </button>
									<button class="dropdown-item"
										onclick={ctx.link().callback(|_| Msg::ChangeContainer(Container::Row))}
									> {"Row"} </button>
									<button class="dropdown-item"
										onclick={ctx.link().callback(|_| Msg::ChangeContainer(Container::Masonry))}
									> {"Masonry"} </button>
								</div>
							</div>
						</div>
					</div>
					<div class="control">
						<div class={classes!("dropdown", if self.show_article_component_dropdown { Some("is-active") } else { None })}>
							<div class="dropdown-trigger">
								<button class="button" onclick={ctx.link().callback(|_| Msg::ToggleArticleComponentDropdown)}>
									<span>{self.article_component.name()}</span>
									<span class="icon is-small">
										<i class="fas fa-angle-down"/>
									</span>
								</button>
							</div>
							<div class="dropdown-menu">
								<div class="dropdown-content">
									<button class="dropdown-item"
										onclick={ctx.link().callback(|_| Msg::ChangeArticleComponent(ArticleComponent::Social))}
									> {"Social"} </button>
									<button class="dropdown-item"
										onclick={ctx.link().callback(|_| Msg::ChangeArticleComponent(ArticleComponent::Gallery))}
									> {"Gallery"} </button>
								</div>
							</div>
						</div>
					</div>
					<div class="control">
						<label class="checkbox">
							<input type="checkbox" checked={self.compact} onclick={ctx.link().callback(|_| Msg::ToggleCompact)}/>
							{ "Compact articles" }
						</label>
					</div>
					<div class="control">
						<label class="label">{"Endpoint"}</label>
						<button class="button" onclick={ctx.link().callback(|_| Msg::SetChooseEndpointModal(true))}>{"Change"}</button>
					</div>
				</div>
			}
		} else {
			html! {}
		}
	}
}