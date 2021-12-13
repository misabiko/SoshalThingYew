use std::rc::Rc;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};

use crate::articles::{SocialArticleData, SocialArticle, sort_by_id};
use crate::endpoints::{EndpointAgent, Request as EndpointRequest, Response as EndpointResponse, TimelineEndpoints};
use crate::containers::{Container, view_container, Props as ContainerProps};

pub struct Timeline {
	articles: Vec<Rc<dyn SocialArticleData>>,	//TODO Use rc::Weak
	options_shown: bool,
	compact: bool,
	endpoint_agent: Box<dyn Bridge<EndpointAgent>>,
	filters: Vec<fn(&Rc<dyn SocialArticleData>) -> bool>,
	container: Container,
	show_container_dropdown: bool,
	column_count: u8,
	width: u8,
}

pub enum Msg {
	Refresh,
	LoadBottom,
	Refreshed(Vec<Rc<dyn SocialArticleData>>),
	RefreshFail,
	EndpointResponse(EndpointResponse),
	ToggleOptions,
	ToggleCompact,
	ChangeContainer(Container),
	ToggleContainerDropdown,
	ChangeColumnCount(u8),
	ChangeWidth(u8),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
	pub name: String,
	#[prop_or_default]
	pub articles: Vec<Rc<dyn SocialArticleData>>,
	#[prop_or_default]
	pub endpoints: Option<TimelineEndpoints>,
	#[prop_or_default]
	pub main_timeline: bool,
	#[prop_or(1)]
	pub column_count: u8
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
						<label class="checkbox">
							<input type="checkbox" checked={self.compact} onclick={ctx.link().callback(|_| Msg::ToggleCompact)}/>
							{ "Compact articles" }
						</label>
					</div>
				</div>
			}
		} else {
			html! {}
		}
	}
}

impl Component for Timeline {
	type Message = Msg;
	type Properties = Props;

	fn create(ctx: &Context<Self>) -> Self {
		let mut endpoint_agent = EndpointAgent::bridge(ctx.link().callback(Msg::EndpointResponse));
		if let Some(endpoints) = ctx.props().endpoints.clone() {
			endpoint_agent.send(EndpointRequest::InitTimeline(endpoints));
		}

		Self {
			articles: ctx.props().articles.clone(),
			options_shown: false,
			compact: false,
			endpoint_agent,
			filters: vec![|a| a.media().len() > 0],
			container: if ctx.props().main_timeline { Container::Row } else { Container::Column },
			show_container_dropdown: false,
			column_count: ctx.props().column_count.clone(),
			width: 1,
		}
	}

	fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::Refresh => {
				self.endpoint_agent.send(EndpointRequest::Refresh);
				false
			}

			Msg::LoadBottom => {
				self.endpoint_agent.send(EndpointRequest::LoadBottom);
				false
			}

			Msg::Refreshed(articles) => {
				self.articles.extend(articles);

				true
			}

			Msg::RefreshFail => false,

			Msg::EndpointResponse(response) =>  {
				match response {
					EndpointResponse::NewArticles(articles) => {
						for a in articles {
							if !self.articles.iter().any(|existing| existing.id() == a.id()) {
								self.articles.push(a);
							}
						}
						true
					}
				}
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

			Msg::ChangeColumnCount(new_column_count) => {
				self.column_count = new_column_count;
				true
			}

			Msg::ChangeWidth(new_width) => {
				self.width = new_width;
				true
			}
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let mut articles = self.articles.clone();
		for filter in &self.filters {
			articles = articles.into_iter().filter(filter).collect();
		}

		articles.sort_by(sort_by_id);

		let style = if self.width > 1 {
			Some(format!("width: {}px", (self.width as i32) * 500))
		} else {
			None
		};
		html! {
			<div class={classes!("timeline", if ctx.props().main_timeline { Some("mainTimeline") } else { None })} {style}>
				<div class="timelineHeader">
					<div class="timelineLeftHeader">
						<strong>{&ctx.props().name}</strong>
					</div>
					<div class="timelineButtons">
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
					articles
				}}) }
			</div>
		}
	}
}