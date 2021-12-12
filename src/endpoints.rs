use std::rc::Rc;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use yew_agent::{Agent, AgentLink, Context, HandlerId};

use crate::error::{Result, Error};
use crate::articles::SocialArticleData;

pub trait Endpoint {
	fn name(&self) -> String;

	fn id(&self) -> &EndpointId;

	fn add_articles(&mut self, articles: Vec<Rc<dyn SocialArticleData>>) {}

	fn refresh(&mut self);

	fn load_top(&mut self);

	fn load_bottom(&mut self);
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
pub enum Msg {
	/// When an endpoint is done refreshing. Contains the endpoint key and articles
	Refreshed(EndpointId, Vec<Rc<dyn SocialArticleData>>),
	RefreshFail(Error),
}

pub enum Request {
	Refresh,
	LoadBottom,
	InitTimeline(TimelineEndpoints),
	AddEndpoint(Box<dyn Fn(EndpointId) -> Box<dyn Endpoint>>),
	FetchResponse(EndpointId, Result<Vec<Rc<dyn SocialArticleData>>>),
	AddArticle(EndpointId, Rc<dyn SocialArticleData>),
}

pub enum Response {
	NewArticles(Vec<Rc<dyn SocialArticleData>>),
}

impl Agent for EndpointAgent {
	type Reach = Context<Self>;
	type Message = Msg;
	type Input = Request;
	type Output = Response;

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
			Msg::Refreshed(endpoint, articles) => {
				self.endpoints.get_mut(&endpoint).unwrap().add_articles(articles.clone());

				for (timeline_id, timeline) in &self.timelines {
					 if timeline.refresh
						.iter()
						.any(|e| e == &endpoint) || timeline.start
						 .iter()
						 .any(|e| e == &endpoint) {
						 log::debug!("Response for timeline");
						 self.link.respond(*timeline_id, Response::NewArticles(articles.clone()));
					 }
				}
			}
			Msg::RefreshFail(err) => {
				log::error!("Failed to fetch \"/proxy/art\"\n{:?}", err);
			}
		}
	}

	fn disconnected(&mut self, id: HandlerId) {
		self.timelines.remove(&id);
	}

	fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
		match msg {
			Request::InitTimeline(endpoints) => {
				self.timelines.insert(id, endpoints);

				for endpoint in &self.timelines[&id].start {
					log::debug!("Refreshing {}", &self.endpoints[&endpoint].name());
					self.endpoints.get_mut(&endpoint).unwrap().refresh();
				}
			}
			Request::AddEndpoint(endpoint) => {
				self.endpoints.insert(self.endpoint_keys, endpoint(self.endpoint_keys));
				self.endpoint_keys += 1;
			}
			Request::Refresh => {
				match self.timelines.get(&id) {
					Some(timeline) => {
						for endpoint_key in &timeline.refresh {
							self.endpoints.get_mut(&endpoint_key).unwrap().refresh();
						}
					}
					None => {
						log::warn!("No TimelineEndpoints found for {:?}", &id);
					}
				}
			}
			Request::LoadBottom => {
				match self.timelines.get(&id) {
					Some(timeline) => {
						for endpoint_key in &timeline.refresh {
							self.endpoints.get_mut(&endpoint_key).unwrap().load_bottom();
						}
					}
					None => {
						log::warn!("No TimelineEndpoints found for {:?}", &id);
					}
				}
			}
			Request::FetchResponse(id, r) => {
				match r {
					Ok(vec_tweets) => self.link.send_message(Msg::Refreshed(id, vec_tweets)),
					Err(err) => self.link.send_message(Msg::RefreshFail(err))
				};
			}
			Request::AddArticle(id, a)
				=> self.link.send_message(Msg::Refreshed(id, vec![a]))
		}
	}
}