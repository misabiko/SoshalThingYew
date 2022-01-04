use std::collections::HashSet;
use yew_agent::{Agent, AgentLink, HandlerId, Context};

pub struct MediaLoadAgent {
	load_queue: HashSet<(String, usize)>,
}

pub enum Request {
	LoadMedia(String, usize)
}

pub enum Response {

}

impl Agent for MediaLoadAgent {
	type Reach = Context<Self>;
	type Message = ();
	type Input = Request;
	type Output = Response;

	fn create(_link: AgentLink<Self>) -> Self {
		Self {
			load_queue: HashSet::new(),
		}
	}

	fn update(&mut self, _msg: Self::Message) {}

	fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
		match msg {
			Request::LoadMedia(article_id, media_index) => {
				self.load_queue.insert((article_id, media_index));
			}
		}
	}
}