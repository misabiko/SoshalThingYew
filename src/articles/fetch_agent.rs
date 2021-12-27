use yew_agent::{Agent, AgentLink, HandlerId, Context as AgentContext, Dispatcher, Dispatched};
use std::collections::HashSet;

pub struct ArticleFetchAgent {
	link: AgentLink<Self>,
	services: HashSet<HandlerId>,
}

pub enum Request {
	RegisterService,
}

pub enum Response {
}

impl Agent for ArticleFetchAgent {
	type Reach = AgentContext<Self>;
	type Message = ();
	type Input = Request;
	type Output = Response;

	fn create(link: AgentLink<Self>) -> Self {
		Self {
			link,
			services: HashSet::new(),
		}
	}

	fn update(&mut self, _msg: Self::Message) {}

	fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
		match msg {
			Request::RegisterService => self.services.insert(id),
		};
	}

	fn disconnected(&mut self, id: HandlerId) {
		self.services.remove(&id);
	}
}