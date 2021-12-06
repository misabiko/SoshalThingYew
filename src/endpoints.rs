use std::{rc::Rc, collections::HashSet};
use yew::worker::*;
use wasm_bindgen::prelude::*;

use crate::articles::SocialArticleData;

pub trait Endpoint {
	fn name(&self) -> String;
	fn refresh(&self);
}

pub struct EndpointAgent {
	link: AgentLink<Self>,
	subscribers: HashSet<HandlerId>,
	endpoints: Vec<Rc<dyn Endpoint>>
}

pub enum AgentRequest {
	//UpdateRateLimit(RateLimit),
	EventBusMsg(String),
}

pub enum AgentOutput {
	//UpdatedRateLimit(RateLimit),
}

pub enum EndpointMsg {
	Refreshed(Vec<Rc<dyn SocialArticleData>>),
	RefreshFail(JsValue),
}

pub enum EndpointRequest {
	Refresh,
	AddEndpoint(Rc<dyn Endpoint>),
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
			subscribers: HashSet::new(),
			endpoints: Vec::new()
		}
	}

	fn update(&mut self, _msg: Self::Message) {}

	fn connected(&mut self, id: HandlerId) {
		self.subscribers.insert(id);
	}

	fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
		match msg {
			EndpointRequest::Refresh => {
				if self.endpoints.len() > 0 {
					self.endpoints[self.endpoints.len() - 1].refresh()
					/*for sub in self.subscribers.iter() {
						self.link.respond(*sub, EndpointResponse::NewArticles(vec![self.endpoints[self.endpoints.len() - 1].get_article()]));
					}*/
				}
			}
			EndpointRequest::AddEndpoint(endpoint) => {
				self.endpoints.push(endpoint)
			}
		}
	}

	fn disconnected(&mut self, id: HandlerId) {
		self.subscribers.remove(&id);
	}
}