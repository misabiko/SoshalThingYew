use std::rc::{Rc, Weak};
use std::collections::{HashMap, HashSet};
use yew::prelude::*;
use yew_agent::AgentLink;
use yew_agent::utils::store::{Store, StoreWrapper};
use std::cell::RefCell;

use crate::error::{Error, FetchResult};
use crate::articles::ArticleData;

#[derive(Clone, Debug, Default)]
pub struct RateLimit {
	pub limit: i32,
	pub remaining: i32,
	pub reset: i32,
}

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

	fn ratelimit(&self) -> Option<&RateLimit> { None }

	fn update_ratelimit(&mut self, _ratelimit: RateLimit) {}

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

pub enum StoreRequest {
	InitTimeline(Rc<RefCell<TimelineEndpoints>>, Callback<Vec<Rc<dyn ArticleData>>>),
	Refresh(Weak<RefCell<TimelineEndpoints>>),
	LoadBottom(Weak<RefCell<TimelineEndpoints>>),
	FetchResponse(RefreshTime, EndpointId, FetchResult<Vec<Rc<dyn ArticleData>>>),
	AddArticles(RefreshTime, EndpointId, Vec<Rc<dyn ArticleData>>),
	AddEndpoint(Box<dyn Fn(EndpointId) -> Box<dyn Endpoint>>),
	InitService(String, Vec<EndpointConstructor>),
	UpdateRateLimit(EndpointId, RateLimit),
}

pub enum Action {
	InitTimeline(Rc<RefCell<TimelineEndpoints>>, Callback<Vec<Rc<dyn ArticleData>>>),
	Refresh(HashSet<EndpointId>),
	LoadBottom(HashSet<EndpointId>),
	Refreshed(RefreshTime, EndpointId, (Vec<Rc<dyn ArticleData>>, Option<RateLimit>)),
	RefreshFail(Error),
	AddEndpoint(Box<dyn Fn(EndpointId) -> Box<dyn Endpoint>>),
	InitService(String, Vec<EndpointConstructor>),
	UpdateRateLimit(EndpointId, RateLimit),
}

type TimelineId = i32;

#[derive(Clone)]
pub struct EndpointConstructor {
	pub name: &'static str,
	pub param_template: Vec<&'static str>,
	pub callback: Rc<dyn Fn(EndpointId, serde_json::Value) -> Box<dyn Endpoint>>
}

pub struct EndpointStore {
	endpoint_counter: EndpointId,
	pub endpoints: HashMap<EndpointId, Box<dyn Endpoint>>,
	timeline_counter: TimelineId,
	pub timelines: HashMap<TimelineId, (Weak<RefCell<TimelineEndpoints>>, Callback<Vec<Rc<dyn ArticleData>>>)>,
	pub services: HashMap<String, Vec<EndpointConstructor>>,
}

impl Store for EndpointStore {
	type Input = StoreRequest;
	type Action = Action;

	fn new() -> Self {
		Self {
			endpoint_counter: i32::MIN,
			endpoints: HashMap::new(),
			timeline_counter: i32::MIN,
			timelines: HashMap::new(),
			services: HashMap::new(),
		}
	}

	fn handle_input(&self, link: AgentLink<StoreWrapper<Self>>, msg: Self::Input) {
		match msg {
			StoreRequest::InitTimeline(endpoints, callback) => link.send_message(Action::InitTimeline(endpoints, callback)),
			StoreRequest::Refresh(endpoints_weak) => {
				let endpoints = endpoints_weak.upgrade().unwrap();
				link.send_message(Action::Refresh(endpoints.borrow().refresh.clone()));
			}
			StoreRequest::LoadBottom(endpoints_weak) => {
				let endpoints = endpoints_weak.upgrade().unwrap();
				link.send_message(Action::LoadBottom(endpoints.borrow().refresh.clone()));
			}
			StoreRequest::FetchResponse(refresh_time, id, response) => {
				match response {
					Ok(response) => link.send_message(Action::Refreshed(refresh_time, id, response)),
					Err(err) => link.send_message(Action::RefreshFail(err))
				};
			}
			StoreRequest::AddArticles(refresh_time, id, articles) =>
				link.send_message(Action::Refreshed(refresh_time, id, (articles, None))),
			StoreRequest::AddEndpoint(endpoint) =>
				link.send_message(Action::AddEndpoint(endpoint)),
			StoreRequest::InitService(name, endpoint_types) =>
				link.send_message(Action::InitService(name, endpoint_types)),
			StoreRequest::UpdateRateLimit(endpoint_id, ratelimit) =>
				link.send_message(Action::UpdateRateLimit(endpoint_id, ratelimit)),
		}
	}

	fn reduce(&mut self, msg: Self::Action) {
		match msg {
			Action::InitTimeline(endpoints, callback) => {
				self.timelines.insert(self.timeline_counter.clone(), (Rc::downgrade(&endpoints), callback));
				self.timeline_counter += 1;

				for endpoint_id in &endpoints.borrow().start {
					log::debug!("Refreshing {}", &self.endpoints[&endpoint_id].name());
					self.endpoints.get_mut(&endpoint_id).unwrap().refresh(RefreshTime::Start);
				}
			}
			Action::Refresh(endpoints) => {
				for endpoint_id in endpoints {
					self.endpoints.get_mut(&endpoint_id).unwrap().refresh(RefreshTime::OnRefresh);
				}
			}
			Action::LoadBottom(endpoints) => {
				for endpoint_id in endpoints {
					self.endpoints.get_mut(&endpoint_id).unwrap().load_bottom(RefreshTime::OnRefresh);
				}
			}
			Action::Refreshed(refresh_time, endpoint_id, response) => {
				log::debug!("{} articles for {}", &response.0.len(), self.endpoints[&endpoint_id].name());
				let endpoint = self.endpoints.get_mut(&endpoint_id).unwrap();
				endpoint.add_articles(response.0.clone());
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
						timeline.1.emit(response.0.clone());
					}
				}
			}
			Action::RefreshFail(err) => {
				log::error!("Failed to fetch:\n{:?}", err);
			}
			Action::AddEndpoint(endpoint) => {
				self.endpoints.insert(self.endpoint_counter, endpoint(self.endpoint_counter));
				self.endpoint_counter += 1;
			}
			Action::InitService(name, endpoint_types) => {
				self.services.insert(name, endpoint_types);
			}
			Action::UpdateRateLimit(endpoint_id, ratelimit) => {
				self.endpoints.get_mut(&endpoint_id).unwrap().update_ratelimit(ratelimit)
			}
		}
	}
}