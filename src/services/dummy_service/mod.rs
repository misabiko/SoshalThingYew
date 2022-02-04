use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use js_sys::Date;
use yew_agent::{Agent, AgentLink, Context, Dispatcher, Dispatched, HandlerId};
use crate::articles::{ArticleData, ArticleMedia};
use crate::EndpointRequest;

use crate::services::endpoint_agent::{EndpointAgent, EndpointConstructors};

static SERVICE_NAME: &'static str = "Dummy Service";

pub struct DummyServiceAgent {
	link: AgentLink<Self>,
	endpoint_agent: Dispatcher<EndpointAgent>,
	articles: HashMap<u32, Rc<RefCell<DummyArticleData>>>,
}

pub enum Msg {}

pub enum Request {}

pub enum Response {}

impl Agent for DummyServiceAgent {
	type Reach = Context<Self>;
	type Message = Msg;
	type Input = Request;
	type Output = Response;

	fn create(link: AgentLink<Self>) -> Self {
		let mut endpoint_agent = EndpointAgent::dispatcher();
		endpoint_agent.send(EndpointRequest::InitService(
			SERVICE_NAME,
			EndpointConstructors {
				endpoint_types: vec![],
				user_endpoint: None,
			}
		));

		Self {
			link,
			endpoint_agent,
			articles: HashMap::new(),
		}
	}

	fn update(&mut self, msg: Self::Message) {
		match msg {}
	}

	fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
		match msg {}
	}
}

#[derive(Clone, Debug)]
pub struct DummyArticleData {
	id: u32,
	creation_time: Date,
	text: String,
	author_name: String,
	author_avatar_url: String,
	author_url: String,
	//media: Vec<ArticleMedia>,
	url: String,
	marked_as_read: bool,
	hidden: bool,
}

impl ArticleData for DummyArticleData {
	fn service(&self) -> &'static str { SERVICE_NAME }

	fn id(&self) -> String { self.id.to_string() }

	fn sortable_id(&self) -> u64 { self.id as u64}

	fn creation_time(&self) -> Date { self.creation_time.clone() }

	fn text(&self) -> String { self.text.clone() }

	fn author_name(&self) -> String { self.author_name.clone() }

	fn author_avatar_url(&self) -> String { self.author_avatar_url.clone() }

	fn author_url(&self) -> String { self.author_url.clone() }

	fn media(&self) -> Vec<ArticleMedia> { vec![] }

	fn url(&self) -> String { self.url.clone() }

	fn marked_as_read(&self) -> bool { self.marked_as_read }

	fn set_marked_as_read(&mut self, value: bool) {
		self.marked_as_read = value;
	}

	fn hidden(&self) -> bool { self.hidden }

	fn set_hidden(&mut self, value: bool) {
		self.hidden = value;
	}

	fn clone_data(&self) -> Box<dyn ArticleData> {
		Box::new(self.clone())
	}

	fn media_loaded(&mut self, _index: usize) {
		log::warn!("Dummy Service doesn't do lazy loading.");
	}
}