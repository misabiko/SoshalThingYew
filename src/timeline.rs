use std::rc::Rc;
use yew::prelude::*;

use crate::articles::{SocialArticleData, SocialArticle};
use crate::endpoints::{EndpointAgent, EndpointRequest, EndpointResponse, TimelineEndpoints};

struct ColumnContainer {
	props: ColumnProps
}

#[derive(Properties, Clone)]
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

	fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
		Self { props }
	}

	fn update(&mut self, _msg: Self::Message) -> ShouldRender {
		false
	}

	fn change(&mut self, props: Self::Properties) -> ShouldRender {
		if self.props != props {
			self.props = props;
			true
		} else {
			false
		}
	}

	fn view(&self) -> Html {
		html! {
			<div class="articlesContainer columnContainer">
				{ for self.props.articles.iter().map(|data| html! {
					<SocialArticle compact=self.props.compact data=data.clone()/>
				})}
			</div>
		}
	}
}

pub struct Timeline {
	link: ComponentLink<Self>,
	props: TimelineProps,
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
	SetCompact(bool),
	//ChangeEndpoint(String),
}


#[derive(Properties, Clone)]
pub struct TimelineProps {
	pub name: String,
	#[prop_or_default]
	pub articles: Vec<Rc<dyn SocialArticleData>>,
	#[prop_or_default]
	pub endpoints: Option<TimelineEndpoints>,
}

impl Timeline {
	fn view_options(&self) -> Html {
		if self.options_shown {
			let update_callback: Callback<bool> = self.link.callback(|checked| TimelineMsg::SetCompact(checked));

			html! {
				<div class="timelineOptions">
					<ybc::Control>
						<ybc::Checkbox name="compact" checked=self.compact update=update_callback>
							{ "Compact articles" }
						</ybc::Checkbox>
					</ybc::Control>
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

	fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
		let mut endpoint_agent = EndpointAgent::bridge(link.callback(TimelineMsg::EndpointResponse));
		if let Some(endpoints) = props.endpoints.clone() {
			endpoint_agent.send(EndpointRequest::InitTimeline(endpoints));
		}

		Self {
			articles: props.articles.clone(),
			props,
			options_shown: false,
			compact: false,
			endpoint_agent,
			link,
		}
	}

	fn update(&mut self, msg: Self::Message) -> ShouldRender {
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

			TimelineMsg::SetCompact(value) => {
				self.compact = value;
				true
			}

			/*TimelineMsg::ChangeEndpoint(new_endpoint) => {
				match new_endpoint.as_str() {
					"Twitter" | "Pixiv" => {
						if Some(&new_endpoint) != self.endpoint.as_ref() {
							self.endpoint = Some(new_endpoint);
							self.articles.clear();

							self.link.send_message(TimelineMsg::Refresh);
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

	fn change(&mut self, _props: Self::Properties) -> ShouldRender {
		false
	}

	fn view(&self) -> Html {
		html! {
			<div class="timeline">
				<div class="timelineHeader">
					<div class="timelineLeftHeader">
						<strong>{&self.props.name}</strong>
					</div>
					<div class="timelineButtons">
						<button onclick=self.link.callback(|_| TimelineMsg::Refresh) title="Refresh">
							<span class="icon">
								<i class="fas fa-sync-alt fa-lg"/>
							</span>
						</button>
						<button onclick=self.link.callback(|_| TimelineMsg::ToggleOptions) title="Expand options">
							<span class="icon">
								<i class="fas fa-ellipsis-v fa-lg"/>
							</span>
						</button>
					</div>
				</div>
				{ self.view_options() }
				<ColumnContainer compact=self.compact articles=self.articles.clone()></ColumnContainer>
			</div>
		}
	}
}