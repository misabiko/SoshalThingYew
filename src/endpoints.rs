use std::rc::Rc;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use yew_agent::{Agent, AgentLink, Context, HandlerId};

use crate::articles::SocialArticleData;

pub trait Endpoint {
	fn name(&self) -> String;

	fn id(&self) -> &EndpointId;

	fn refresh(&mut self);
}

pub type EndpointId = i32;

#[derive(Clone, PartialEq)]
pub struct TimelineEndpoints {
	pub start: Vec<EndpointId>,
	pub refresh: Vec<EndpointId>,
}

pub struct EndpointAgent {
	link: AgentLink<Self>,
	endpoint_keys: EndpointId,
	endpoints: HashMap<EndpointId, Box<dyn Endpoint>>,
	timelines: HashMap<HandlerId, TimelineEndpoints>,
}

//TODO Use struct variants instead of tuples?
pub enum EndpointMsg {
	/// When an endpoint is done refreshing. Contains the endpoint key and articles
	Refreshed(EndpointId, Vec<Rc<dyn SocialArticleData>>),
	RefreshFail(JsValue),
}

pub enum EndpointRequest {
	Refresh,
	InitTimeline(TimelineEndpoints),
	AddEndpoint(Box<dyn Fn(EndpointId) -> Box<dyn Endpoint>>),
	FetchResponse(EndpointId, Result<Vec<Rc<dyn SocialArticleData>>, JsValue>),
	AddArticle(EndpointId, Rc<dyn SocialArticleData>),
}

pub enum EndpointResponse {
	NewArticles(Vec<Rc<dyn SocialArticleData>>),
}

impl Agent for EndpointAgent {
	type Reach = Context<Self>;
	type Message = EndpointMsg;
	type Input = EndpointRequest;
	type Output = EndpointResponse;

	fn create(link: AgentLink<Self>) -> Self {
		Self {
			link,
			endpoint_keys: i32::MIN,
			endpoints: HashMap::new(),
			timelines: HashMap::new(),
		}
	}

	fn update(&mut self, msg: Self::Message) {
		match msg {
			EndpointMsg::Refreshed(endpoint, articles) => {
				for (timeline_id, timeline) in &self.timelines {
					timeline.refresh
						.iter()
						.find(|e| *e == &endpoint);

					log::debug!("Response for timeline");
					self.link.respond(*timeline_id, EndpointResponse::NewArticles(articles.clone()));
				}
			}
			EndpointMsg::RefreshFail(err) => {
				log::error!("Failed to fetch \"/proxy/art\"\n{:?}", err);
			}
		}
	}

	fn disconnected(&mut self, id: HandlerId) {
		self.timelines.remove(&id);
	}

	fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
		match msg {
			EndpointRequest::InitTimeline(endpoints) => {
				self.timelines.insert(id, endpoints);

				for endpoint in &self.timelines[&id].start {
					log::debug!("Refreshing {}", &self.endpoints[&endpoint].name());
					self.endpoints.get_mut(&endpoint).unwrap().refresh();
				}
			}
			EndpointRequest::AddEndpoint(endpoint) => {
				self.endpoints.insert(self.endpoint_keys, endpoint(self.endpoint_keys));
				self.endpoint_keys += 1;
			}
			EndpointRequest::Refresh => {
				match self.timelines.get(&id) {
					Some(timeline) => {
						for endpoint_key in &self.timelines[&id].refresh {
							self.endpoints.get_mut(&endpoint_key).unwrap().refresh();
						}
					}
					None => {
						log::warn!("No TimelineEndpoints found for {:?}", &id);
					}
				}
			}
			EndpointRequest::FetchResponse(id, r) => {
				match r {
					Ok(vec_tweets) => self.link.send_message(EndpointMsg::Refreshed(id, vec_tweets)),
					Err(err) => self.link.send_message(EndpointMsg::RefreshFail(err))
				};
			}
			EndpointRequest::AddArticle(id, a)
				=> self.link.send_message(EndpointMsg::Refreshed(id, vec![a]))
		}
	}
}