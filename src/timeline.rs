use std::rc::{Rc, Weak};
use std::cell::RefCell;
use yew::prelude::*;
use yew_agent::Bridge;
use yew_agent::utils::store::{StoreWrapper, ReadOnly, Bridgeable};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use rand::{thread_rng, seq::SliceRandom};

use crate::articles::{ArticleComponent, ArticleData, sort_by_id};
use crate::services::endpoints::{EndpointStore, TimelineEndpoints, Request as EndpointRequest};
use crate::containers::{Container, view_container, Props as ContainerProps};
use crate::modals::Modal;
use crate::choose_endpoints::ChooseEndpoints;
use crate::dropdown::{Dropdown, DropdownLabel};

pub struct Timeline {
	endpoints: Rc<RefCell<TimelineEndpoints>>,
	articles: Vec<Weak<RefCell<dyn ArticleData>>>,
	options_shown: bool,
	compact: bool,
	endpoint_store: Box<dyn Bridge<StoreWrapper<EndpointStore>>>,
	filters: Vec<fn(&Weak<RefCell<dyn ArticleData>>) -> bool>,
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
	Refreshed(Vec<Weak<RefCell<dyn ArticleData>>>),
	RefreshFail,
	NewArticles(Vec<Weak<RefCell<dyn ArticleData>>>),
	ClearArticles,
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

#[derive(Properties, Clone)]
pub struct Props {
	pub name: String,
	#[prop_or_default]
	pub articles: Vec<Weak<RefCell<dyn ArticleData>>>,
	#[prop_or_default]
	pub endpoints: Option<TimelineEndpoints>,
	#[prop_or_default]
	pub main_timeline: bool,
	#[prop_or(1)]
	pub column_count: u8,
	#[prop_or_default]
	pub children: Children,
}

impl PartialEq for Props {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name &&
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
		let endpoints = match ctx.props().endpoints.clone() {
			Some(endpoints) => Rc::new(RefCell::new(endpoints)),
			None => Rc::new(RefCell::new(TimelineEndpoints::default()))
		};

		let mut endpoint_store = EndpointStore::bridge(ctx.link().callback(Msg::EndpointStoreResponse));
		endpoint_store.send(EndpointRequest::InitTimeline(endpoints.clone(), ctx.link().callback(Msg::NewArticles)));

		Self {
			endpoints,
			articles: ctx.props().articles.clone(),
			options_shown: false,
			compact: false,
			endpoint_store,
			filters: vec![|a| {
				match a.upgrade() {
					Some(strong) => strong.borrow().media().len() > 0,
					None => false,
				}
			}],
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
						true => html! {
							<Modal modal_title="Choose Endpoints" close_modal_callback={ctx.link().callback(|_| Msg::SetChooseEndpointModal(false))}>
								<ChooseEndpoints
									timeline_endpoints={Rc::downgrade(&self.endpoints)}
								/>
							</Modal>
						},
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
						<Dropdown current_label={DropdownLabel::Text(self.container.name().to_string())}>
							<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ChangeContainer(Container::Column))}> {"Column"} </a>
							<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ChangeContainer(Container::Row))}> {"Row"} </a>
							<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ChangeContainer(Container::Masonry))}> {"Masonry"} </a>
						</Dropdown>
					</div>
					<div class="control">
						<Dropdown current_label={DropdownLabel::Text(self.article_component.name().to_string())}>
							<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ChangeArticleComponent(ArticleComponent::Social))}> {"Social"} </a>
							<a class="dropdown-item" onclick={ctx.link().callback(|_| Msg::ChangeArticleComponent(ArticleComponent::Gallery))}> {"Gallery"} </a>
						</Dropdown>
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
					<div class="control">
						<button class="button" onclick={ctx.link().callback(|_| Msg::ClearArticles)}>{"Clear Articles"}</button>
					</div>
				</div>
			}
		} else {
			html! {}
		}
	}
}