use std::rc::{Rc, Weak};
use std::collections::HashMap;
use yew::prelude::*;
use yew_agent::{Agent, AgentLink, Context, HandlerId};
use yew_agent::utils::store::{Store, StoreWrapper};

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

#[derive(Clone, PartialEq)]
pub enum RefreshTime {
	Start,
	OnRefresh,
}

pub enum StoreRequest {
	InitTimeline(Rc<TimelineEndpoints>, Callback<Vec<Rc<dyn ArticleData>>>),
	Refresh(Weak<TimelineEndpoints>, Callback<Vec<Rc<dyn ArticleData>>>),
	LoadBottom(Weak<TimelineEndpoints>, Callback<Vec<Rc<dyn ArticleData>>>),
	FetchResponse(EndpointId, Result<Vec<Rc<dyn ArticleData>>>),
	AddArticles(EndpointId, Vec<Rc<dyn ArticleData>>),
	AddEndpoint(Box<dyn Fn(EndpointId) -> Box<dyn Endpoint>>),
}

pub enum Action {
	InitTimeline(Rc<TimelineEndpoints>, Callback<Vec<Rc<dyn ArticleData>>>),
	Refresh(Vec<EndpointId>),
	LoadBottom(Vec<EndpointId>),
	Refreshed(EndpointId, Vec<Rc<dyn ArticleData>>),
	RefreshFail(Error),
	AddEndpoint(Box<dyn Fn(EndpointId) -> Box<dyn Endpoint>>),
}

type TimelineId = i32;

pub struct EndpointStore {
	endpoint_counter: EndpointId,
	pub endpoints: HashMap<EndpointId, Box<dyn Endpoint>>,
	timeline_counter: TimelineId,
	pub timelines: HashMap<TimelineId, (Weak<TimelineEndpoints>, Callback<Vec<Rc<dyn ArticleData>>>)>,
}

impl Store for EndpointStore {
	type Action = Action;
	type Input = StoreRequest;

	fn new() -> Self {
		Self {
			endpoint_counter: i32::MIN,
			endpoints: HashMap::new(),
			timeline_counter: i32::MIN,
			timelines: HashMap::new(),
		}
	}

	fn handle_input(&self, link: AgentLink<StoreWrapper<Self>>, msg: Self::Input) {
		match msg {
			StoreRequest::InitTimeline(endpoints, callback) => link.send_message(Action::InitTimeline(endpoints, callback)),
			StoreRequest::Refresh(endpoints_weak, callback) => {
				let endpoints = endpoints_weak.upgrade().unwrap();
				link.send_message(Action::Refresh(endpoints.refresh.clone()));
			}
			StoreRequest::LoadBottom(endpoints_weak, callback) => {
				let endpoints = endpoints_weak.upgrade().unwrap();
				link.send_message(Action::LoadBottom(endpoints.refresh.clone()));
			}
			StoreRequest::FetchResponse(id, response) => {
				match response {
					Ok(vec_tweets) => link.send_message(Action::Refreshed(id, vec_tweets)),
					Err(err) => link.send_message(Action::RefreshFail(err))
				};
			}
			StoreRequest::AddArticles(id, articles) =>
				link.send_message(Action::Refreshed(id, articles)),
			StoreRequest::AddEndpoint(endpoint) =>
				link.send_message(Action::AddEndpoint(endpoint)),
		}
	}

	fn reduce(&mut self, msg: Self::Action) {
		match msg {
			Action::InitTimeline(endpoints, callback) => {
				self.timelines.insert(self.timeline_counter.clone(), (Rc::downgrade(&endpoints), callback));
				self.timeline_counter += 1;

				for endpoint_id in &endpoints.start {
					log::debug!("Refreshing {}", &self.endpoints[&endpoint_id].name());
					self.endpoints.get_mut(&endpoint_id).unwrap().refresh();
				}
			}
			Action::Refresh(endpoints) => {
				for endpoint_id in endpoints {
					self.endpoints.get_mut(&endpoint_id).unwrap().refresh();
				}
			}
			Action::LoadBottom(endpoints) => {
				for endpoint_id in endpoints {
					self.endpoints.get_mut(&endpoint_id).unwrap().load_bottom();
				}
			}
			Action::Refreshed(endpoint_id, articles) => {
				log::debug!("{} articles for {}", &articles.len(), self.endpoints[&endpoint_id].name());
				self.endpoints.get_mut(&endpoint_id).unwrap().add_articles(articles.clone());

				for (timeline_id, timeline) in &self.timelines {
					//TODO Add RefreshTime enum
					let timeline_strong = timeline.0.upgrade().unwrap();
					if timeline_strong.refresh.iter().any(|e| e == &endpoint_id) ||
						timeline_strong.start.iter().any(|e| e == &endpoint_id) {
						timeline.1.emit(articles.clone());
					}
				}
			}
			Action::RefreshFail(err) => {
				log::error!("Failed to fetch \"/proxy/art\"\n{:?}", err);
			}
			Action::AddEndpoint(endpoint) => {
				self.endpoints.insert(self.endpoint_counter, endpoint(self.endpoint_counter));
				self.endpoint_counter += 1;
			}
		}
	}
}