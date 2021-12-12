use std::rc::Rc;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::articles::{SocialArticleData, SocialArticle};
use crate::endpoints::{EndpointAgent, Request as EndpointRequest, Response as EndpointResponse, TimelineEndpoints};

struct ColumnContainer;

#[derive(Properties)]
struct ColumnProps {
	compact: bool,
	articles: Vec<Rc<dyn SocialArticleData>>
}

impl PartialEq for ColumnProps {
	fn eq(&self, other: &Self) -> bool {
		self.compact == other.compact &&
		self.articles.len() == other.articles.len() &&
		!self.articles.iter().zip(other.articles.iter())
			.any(|(ai, bi)| !Rc::ptr_eq(&ai, &bi))
	}
}

impl Component for ColumnContainer {
	type Message = ();
	type Properties = ColumnProps;

	fn create(_ctx: &Context<Self>) -> Self {
		Self {}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		html! {
			<div class="articlesContainer columnContainer">
				{ for ctx.props().articles.iter().map(|data| html! {
					<SocialArticle compact={ctx.props().compact} data={data.clone()}/>
				})}
			</div>
		}
	}
}

pub struct Timeline {
	articles: Vec<Rc<dyn SocialArticleData>>,	//TODO Use rc::Weak
	options_shown: bool,
	compact: bool,
	endpoint_agent: Box<dyn Bridge<EndpointAgent>>,
}

pub enum TimelineMsg {
	Refresh,
	Refreshed(Vec<Rc<dyn SocialArticleData>>),
	RefreshFail,
	EndpointResponse(EndpointResponse),
	ToggleOptions,
	ToggleCompact,
	//ChangeEndpoint(String),
}


#[derive(Properties, Clone, PartialEq)]
pub struct TimelineProps {
	pub name: String,
	#[prop_or_default]
	pub articles: Vec<Rc<dyn SocialArticleData>>,
	#[prop_or_default]
	pub endpoints: Option<TimelineEndpoints>,
}

impl Timeline {
	fn view_options(&self, ctx: &Context<Self>) -> Html {
		if self.options_shown {
			html! {
				<div class="timelineOptions">
					<div class="control">
						<label class="checkbox">
							<input checked={self.compact} onclick={ctx.link().callback(|_| TimelineMsg::ToggleCompact)}/>
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
	type Message = TimelineMsg;
	type Properties = TimelineProps;

	fn create(ctx: &Context<Self>) -> Self {
		let mut endpoint_agent = EndpointAgent::bridge(ctx.link().callback(TimelineMsg::EndpointResponse));
		if let Some(endpoints) = ctx.props().endpoints.clone() {
			endpoint_agent.send(EndpointRequest::InitTimeline(endpoints));
		}

		Self {
			articles: ctx.props().articles.clone(),
			options_shown: false,
			compact: false,
			endpoint_agent,
		}
	}

	fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			TimelineMsg::Refresh => {
				/*match self.endpoint.as_ref().map(|e| e.as_str()) {
					Some("Twitter") =>  {
						log::debug!("Trying to refresh Twitter");
						self.endpoint_agent.send(EndpointRequest::Refresh);
						true
					}
					Some("Pixiv") => {
						log::debug!("Trying to refresh Pixiv");
						self.endpoint_agent.send(EndpointRequest::Refresh);
						true
					}
					_ => false
				}*/

				log::debug!("Timeline Refresh!");
				self.endpoint_agent.send(EndpointRequest::Refresh);
				false
			}

			TimelineMsg::Refreshed(articles) => {
				self.articles.extend(articles);

				true
			}

			TimelineMsg::RefreshFail => false,

			TimelineMsg::EndpointResponse(response) =>  {
				match response {
					EndpointResponse::NewArticles(articles) => {
						self.articles.extend(articles);
						true
					}
					_ => false
				}
			}

			TimelineMsg::ToggleOptions => {
				self.options_shown = !self.options_shown;
				true
			}

			TimelineMsg::ToggleCompact => {
				self.compact = !self.compact;
				true
			}

			/*TimelineMsg::ChangeEndpoint(new_endpoint) => {
				match new_endpoint.as_str() {
					"Twitter" | "Pixiv" => {
						if Some(&new_endpoint) != self.endpoint.as_ref() {
							self.endpoint = Some(new_endpoint);
							self.articles.clear();

							ctx.link().send_message(TimelineMsg::Refresh);
							true
						}else {
							false
						}
					}
					_ => false
				}
			}*/
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		html! {
			<div class="timeline">
				<div class="timelineHeader">
					<div class="timelineLeftHeader">
						<strong>{&ctx.props().name}</strong>
					</div>
					<div class="timelineButtons">
						<button onclick={ctx.link().callback(|_| TimelineMsg::Refresh)} title="Refresh">
							<span class="icon">
								<i class="fas fa-sync-alt fa-lg"/>
							</span>
						</button>
						<button onclick={ctx.link().callback(|_| TimelineMsg::ToggleOptions)} title="Expand options">
							<span class="icon">
								<i class="fas fa-ellipsis-v fa-lg"/>
							</span>
						</button>
					</div>
				</div>
				{ self.view_options(ctx) }
				<ColumnContainer compact={self.compact} articles={self.articles.clone()}></ColumnContainer>
			</div>
		}
	}
}