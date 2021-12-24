use std::rc::{Rc, Weak};
use std::collections::{HashMap, HashSet};
use yew::prelude::*;
use yew_agent::{Agent, Context as AgentContext, AgentLink, HandlerId};
use std::cell::RefCell;
use reqwest::header::HeaderMap;

use crate::error::{Error, FetchResult};
use crate::articles::ArticleData;
use crate::timeline::agent::TimelineEndpointsStorage;
use crate::timeline::sort_methods::sort_by_id;
use crate::{TimelineId, TimelinePropsEndpointsClosure};

#[derive(Clone, Debug)]
pub struct RateLimit {
	pub limit: i32,
	pub remaining: i32,
	pub reset: i32,
}

impl Default for RateLimit {
	fn default() -> Self {
		Self {
			limit: 1,
			remaining: 1,
			reset: 0,
		}
	}
}

impl TryFrom<&HeaderMap> for RateLimit {
	type Error = Error;

	fn try_from(headers: &HeaderMap) -> std::result::Result<Self, Self::Error> {
		let ret = Self {
			limit: headers.get("x-rate-limit-limit")
				.ok_or(Error::from("Couldn't get ratelimit's limit"))?
				.to_str()?
				.parse()?,
			remaining: headers.get("x-rate-limit-remaining")
				.ok_or(Error::from("Couldn't get ratelimit's remaining"))?
				.to_str()?
				.parse()?,
			reset: headers.get("x-rate-limit-reset")
				.ok_or(Error::from("Couldn't get ratelimit's reset"))?
				.to_str()?
				.parse()?,
		};
		Ok(ret)
	}
}

impl RateLimit {
	pub fn can_refresh(&mut self) -> bool {
		if (self.reset as f64) < js_sys::Date::now() {
			self.remaining = self.limit;
			true
		}else {
			self.remaining > 0
		}
	}
}

pub trait Endpoint {
	fn name(&self) -> String;

	fn id(&self) -> &EndpointId;

	fn articles(&mut self) -> &mut Vec<Weak<RefCell<dyn ArticleData>>>;

	fn add_articles(&mut self, articles: Vec<Weak<RefCell<dyn ArticleData>>>)  {
		for a in articles {
			if !self.articles().iter().any(|existing| Weak::ptr_eq(&existing, &a)) {
				self.articles().push(a);
			}
		}
		self.articles().sort_by(sort_by_id)
	}

	fn ratelimit(&self) -> Option<&RateLimit> { None }

	fn get_mut_ratelimit(&mut self) -> Option<&mut RateLimit> { None }

	fn update_ratelimit(&mut self, _ratelimit: RateLimit) {}

	fn can_refresh(&self) -> bool { true }

	fn refresh(&mut self, refresh_time: RefreshTime);

	fn load_top(&mut self, refresh_time: RefreshTime) {
		log::debug!("{} doesn't implement load_top()", self.name());
		self.refresh(refresh_time)
	}

	fn load_bottom(&mut self, refresh_time: RefreshTime) {
		log::debug!("{} doesn't implement load_bottom()", self.name());
		self.refresh(refresh_time)
	}
}

pub type EndpointId = i32;

//Maybe HashMap<RefreshTime, HashSet<EndpointId>> ?
#[derive(Clone, PartialEq, Default)]
pub struct TimelineEndpoints {
	pub start: HashSet<EndpointId>,
	pub refresh: HashSet<EndpointId>,
}

#[derive(Clone, PartialEq)]
pub enum RefreshTime {
	Start,
	OnRefresh,
}

pub enum Msg {
	Refreshed(RefreshTime, EndpointId, (Vec<Rc<RefCell<dyn ArticleData>>>, Option<RateLimit>)),
	RefreshFail(Error),
	UpdatedState,
}

pub enum Request {
	InitTimeline(TimelineId, Rc<RefCell<TimelineEndpoints>>, Callback<Vec<Weak<RefCell<dyn ArticleData>>>>),
	RemoveTimeline(TimelineId),
	Refresh(Weak<RefCell<TimelineEndpoints>>),
	LoadBottom(Weak<RefCell<TimelineEndpoints>>),
	EndpointFetchResponse(RefreshTime, EndpointId, FetchResult<Vec<Rc<RefCell<dyn ArticleData>>>>),
	AddArticles(RefreshTime, EndpointId, Vec<Rc<RefCell<dyn ArticleData>>>),
	AddEndpoint(Box<dyn Fn(EndpointId) -> Box<dyn Endpoint>>),
	InitService(String, EndpointConstructors),
	UpdateRateLimit(EndpointId, RateLimit),
	BatchNewEndpoints(Vec<(TimelineEndpointsStorage, TimelinePropsEndpointsClosure)>),
	RegisterTimelineContainer,
}

pub enum Response {
	UpdatedState(HashMap<String, EndpointConstructors>, Vec<EndpointView>),
	BatchRequestResponse(Vec<(TimelineEndpoints, TimelinePropsEndpointsClosure)>)
}

#[derive(Clone)]
pub struct EndpointConstructor {
	pub name: &'static str,
	pub param_template: Vec<&'static str>,
	pub callback: Rc<dyn Fn(EndpointId, serde_json::Value) -> Box<dyn Endpoint>>
}

#[derive(Clone)]
pub struct EndpointConstructors {
	pub endpoint_types: Vec<EndpointConstructor>,
	pub user_endpoint: Option<usize>,
}

pub struct EndpointView {
	pub id: EndpointId,
	pub name: String,
	pub ratelimit: Option<RateLimit>,
}

pub struct EndpointAgent {
	link: AgentLink<Self>,
	endpoint_counter: EndpointId,
	pub endpoints: HashMap<EndpointId, Box<dyn Endpoint>>,
	pub timelines: HashMap<TimelineId, (Weak<RefCell<TimelineEndpoints>>, Callback<Vec<Weak<RefCell<dyn ArticleData>>>>)>,
	pub services: HashMap<String, EndpointConstructors>,
	subscribers: HashSet<HandlerId>,
	timeline_container: Option<HandlerId>,
}

impl Agent for EndpointAgent {
	type Reach = AgentContext<Self>;
	type Message = Msg;
	type Input = Request;
	type Output = Response;

	fn create(link: AgentLink<Self>) -> Self {
		Self {
			link,
			endpoint_counter: i32::MIN,
			endpoints: HashMap::new(),
			timelines: HashMap::new(),
			services: HashMap::new(),
			subscribers: HashSet::new(),
			timeline_container: None,
		}
	}

	fn update(&mut self, msg: Self::Message) {
		match msg {
			Msg::Refreshed(refresh_time, endpoint_id, response) => {
				log::debug!("{} articles for {}", &response.0.len(), self.endpoints[&endpoint_id].name());
				let endpoint = self.endpoints.get_mut(&endpoint_id).unwrap();
				endpoint.add_articles(response.0.iter().map(|article| Rc::downgrade(&article)).collect());
				if let Some(ratelimit) = response.1 {
					endpoint.update_ratelimit(ratelimit);
				}

				for (_timeline_id, timeline) in &self.timelines {
					let timeline_strong = timeline.0.upgrade().unwrap();
					let borrow = timeline_strong.borrow();
					let endpoints = match &refresh_time {
						RefreshTime::OnRefresh => &borrow.refresh,
						RefreshTime::Start => &borrow.start,
					};

					if endpoints.iter().any(|e| e == &endpoint_id) {
						timeline.1.emit(response.0.iter().map(|article| Rc::downgrade(&article)).collect());
					}
				}

				self.link.send_message(Msg::UpdatedState);
			}
			Msg::RefreshFail(err) => {
				log::error!("Failed to fetch:\n{:?}", err);
			}
			Msg::UpdatedState => {
				for sub in &self.subscribers {
					if sub.is_respondable() {
						self.link.respond(*sub, Response::UpdatedState(self.services.clone(), self.endpoints.iter().map(|(id, e)| EndpointView {
							id: id.clone(),
							name: e.name(),
							ratelimit: e.ratelimit().cloned(),
						}).collect()));
					}
				}
			}
		}
	}

	fn connected(&mut self, id: HandlerId) {
		self.subscribers.insert(id);
	}

	fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
		match msg {
			Request::InitTimeline(id, endpoints, callback) => {
				self.timelines.insert(id, (Rc::downgrade(&endpoints), callback));

				for endpoint_id in &endpoints.borrow().start {
					let endpoint = self.endpoints.get_mut(&endpoint_id).unwrap();
					if endpoint.get_mut_ratelimit().map(|r| r.can_refresh()).unwrap_or(true) {
						endpoint.refresh(RefreshTime::Start);
					}else {
						log::warn!("Can't refresh {}", &endpoint.name());
					}
				}
			},
			Request::RemoveTimeline(id) => {
				self.timelines.remove(&id);
			}
			Request::Refresh(endpoints_weak) => {
				let endpoints = endpoints_weak.upgrade().unwrap();
				for endpoint_id in endpoints.borrow().refresh.clone() {
					let endpoint = self.endpoints.get_mut(&endpoint_id).unwrap();
					if endpoint.get_mut_ratelimit().map(|r| r.can_refresh()).unwrap_or(true) {
						endpoint.refresh(RefreshTime::OnRefresh);
					}else {
						log::warn!("Can't refresh {}", &endpoint.name());
					}
				}
			}
			Request::LoadBottom(endpoints_weak) => {
				let endpoints = endpoints_weak.upgrade().unwrap();
				for endpoint_id in endpoints.borrow().refresh.clone() {
					let endpoint = self.endpoints.get_mut(&endpoint_id).unwrap();
					if endpoint.get_mut_ratelimit().map(|r| r.can_refresh()).unwrap_or(true) {
						endpoint.load_bottom(RefreshTime::OnRefresh);
					}else {
						log::warn!("Can't refresh {}", &endpoint.name());
					}
				}
			}
			Request::EndpointFetchResponse(refresh_time, endpoint_id, response) => {
				match response {
					Ok(response) => self.link.send_message(Msg::Refreshed(refresh_time, endpoint_id, response)),
					Err(err) => self.link.send_message(Msg::RefreshFail(err)),
				};
			}
			Request::AddArticles(refresh_time, id, articles) =>
				self.link.send_message(Msg::Refreshed(refresh_time, id, (articles, None))),
			Request::AddEndpoint(endpoint) => {
				self.endpoints.insert(self.endpoint_counter, endpoint(self.endpoint_counter));
				self.endpoint_counter += 1;

				self.link.send_message(Msg::UpdatedState);
			},
			Request::InitService(name, endpoints) => {
				self.services.insert(name, endpoints);

				self.link.send_message(Msg::UpdatedState);
			},
			Request::UpdateRateLimit(endpoint_id, ratelimit) => {
				self.endpoints.get_mut(&endpoint_id).unwrap().update_ratelimit(ratelimit)
			},
			Request::BatchNewEndpoints(timelines) => {
				let endpoints: Vec<(TimelineEndpoints, TimelinePropsEndpointsClosure)> = timelines.into_iter().map(|(constructor, callback)| {
					let start = constructor.start.iter().map(|e| {
						log::debug!("services {:?}", self.services.keys());
						log::debug!("{} endpoints for service {}", self.services[&e.service].endpoint_types.len(), &e.service);
						let constructor = self.services[&e.service].endpoint_types[e.endpoint_type.clone()].clone();
						let params = e.params.clone();

						let id = self.endpoint_counter.clone();
						self.endpoints.insert(self.endpoint_counter, (constructor.callback)(id, params.clone()));
						self.endpoint_counter += 1;

						id
					}).collect::<HashSet<EndpointId>>();

					let refresh = constructor.refresh.iter().map(|e| {
						let constructor = self.services[&e.service].endpoint_types[e.endpoint_type.clone()].clone();
						let params = e.params.clone();

						let id = self.endpoint_counter.clone();
						self.endpoints.insert(self.endpoint_counter, (constructor.callback)(id, params.clone()));
						self.endpoint_counter += 1;

						id
					}).collect::<HashSet<EndpointId>>();

					(TimelineEndpoints { start, refresh }, callback)
				}).collect();


				if let Some(timeline_container) = self.timeline_container {
					self.link.respond(timeline_container, Response::BatchRequestResponse(endpoints));
				}
			},
			Request::RegisterTimelineContainer => self.timeline_container = Some(id),
		}
	}

	fn disconnected(&mut self, id: HandlerId) {
		self.subscribers.remove(&id);

		if self.timeline_container == Some(id) {
			self.timeline_container = None
		}
	}
}