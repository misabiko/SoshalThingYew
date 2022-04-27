use std::collections::HashMap;
use yew_agent::{Agent, Context as AgentContext, AgentLink, HandlerId};

pub struct ModalAgent {
	link: AgentLink<Self>,
	subscribers: HashMap<ModalType, HandlerId>
}

pub enum ModalRequest {
	Register(ModalType),
	ActivateModal(ModalType),
}

impl Agent for ModalAgent {
	type Reach = AgentContext<Self>;
	type Message = ();
	type Input = ModalRequest;
	type Output = ModalType;

	fn create(link: AgentLink<Self>) -> Self {
		Self {
			link,
			subscribers: HashMap::new(),
		}
	}

	fn update(&mut self, _msg: Self::Message) {}

	fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
		match msg {
			ModalRequest::Register(modal) => {
				self.subscribers.insert(modal, id);
			}
			ModalRequest::ActivateModal(modal) => self.link.respond(self.subscribers[&modal], modal),
		}
	}

	fn disconnected(&mut self, id: HandlerId) {
		self.subscribers.retain(|_, subscriber| *subscriber != id);
	}
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum ModalType {
	BatchAction,
}