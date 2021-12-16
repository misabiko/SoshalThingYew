use std::rc::{Rc, Weak};
use std::collections::HashMap;
use yew_agent::{Agent, AgentLink, Context, HandlerId};

use crate::error::{Result, Error};
use crate::articles::ArticleData;

pub trait Endpoint {
	fn name(&self) -> String;

	fn id(&self) -> &EndpointId;

	fn articles(&mut self) -> &mut Vec<Rc<dyn ArticleData>>;

	fn add_articles(&mut self, articles: Vec<Rc<dyn ArticleData>>)  {
		for a in articles {
			if !self.articles().iter().any(|existing| existing.id() == a.id()) {
				self.articles().push(a);
			}
		}
		self.articles().sort_by(|a, b| b.id().partial_cmp(&a.id()).unwrap())
	}

	fn refresh(&mut self);

	fn load_top(&mut self) {
		log::debug!("{} doesn't implement load_top()", self.name());
		self.refresh()
	}

	fn load_bottom(&mut self) {
		log::debug!("{} doesn't implement load_bottom()", self.name());
		self.refresh()
	}
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
	timelines: HashMap<HandlerId, Weak<TimelineEndpoints>>,
}

pub enum Msg {
	Refreshed(EndpointId, Vec<Rc<dyn ArticleData>>),
	RefreshFail(Error),
}

pub enum Request {
	Refresh,
	LoadBottom,
	InitTimeline(Rc<TimelineEndpoints>),
	AddEndpoint(Box<dyn Fn(EndpointId) -> Box<dyn Endpoint>>),
	FetchResponse(EndpointId, Result<Vec<Rc<dyn ArticleData>>>),
	AddArticles(EndpointId, Vec<Rc<dyn ArticleData>>),
}

pub enum Response {
	NewArticles(Vec<Rc<dyn ArticleData>>),
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
			Msg::Refreshed(endpoint_id, articles) => {
				log::debug!("{} articles for {}", &articles.len(), self.endpoints[&endpoint_id].name());
				self.endpoints.get_mut(&endpoint_id).unwrap().add_articles(articles.clone());

				for (timeline_id, timeline_weak) in &self.timelines {
					match timeline_weak.upgrade() {
						Some(strong) => {
							if strong.refresh.iter().any(|e| e == &endpoint_id) ||
								strong.start.iter().any(|e| e == &endpoint_id) {
								self.link.respond(*timeline_id, Response::NewArticles(articles.clone()));
							}
						}
						None => log::warn!("Couldn't upgrade timeline endpoints for {:?}", &timeline_id)
					};
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
				self.timelines.insert(id, Rc::downgrade(&endpoints));

				for endpoint_id in &endpoints.start {
					log::debug!("Refreshing {}", &self.endpoints[&endpoint_id].name());
					self.endpoints.get_mut(&endpoint_id).unwrap().refresh();
				}
			}
			Request::AddEndpoint(endpoint) => {
				self.endpoints.insert(self.endpoint_keys, endpoint(self.endpoint_keys));
				self.endpoint_keys += 1;
			}
			Request::Refresh => {
				match self.timelines.get(&id).and_then(Weak::upgrade) {
					Some(timeline) => {
						for endpoint_id in &timeline.refresh {
							self.endpoints.get_mut(&endpoint_id).unwrap().refresh();
						}
					}
					None => {
						log::warn!("No TimelineEndpoints found for {:?}", &id);
					}
				}
			}
			Request::LoadBottom => {
				match self.timelines.get(&id).and_then(Weak::upgrade) {
					Some(timeline) => {
						for endpoint_id in &timeline.refresh {
							self.endpoints.get_mut(&endpoint_id).unwrap().load_bottom();
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
			Request::AddArticles(id, articles)
				=> self.link.send_message(Msg::Refreshed(id, articles))
		}
	}
}