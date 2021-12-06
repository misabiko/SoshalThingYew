use std::{rc::Rc, collections::HashSet};
use yew::agent::Dispatched;
use yew::worker::*;

use crate::articles::SocialArticleData;
use crate::endpoints::{EndpointAgent, Endpoint, EndpointRequest};

pub struct PixivArticleData {
	id: String
}

impl SocialArticleData for PixivArticleData {
	fn id(&self) -> String {
		self.id.clone()
	}
	fn text(&self) -> String {
		"同じキャラ描きまくってる".to_owned()
	}
	fn author_username(&self) -> String {
		"1283639".to_owned()
	}
	fn author_name(&self) -> String {
		"Aまみん".to_owned()
	}
	fn author_avatar_url(&self) -> String {
		"https://i.pximg.net/user-profile/img/2021/05/09/18/17/27/20672817_97cf645014317d5432bc5cc946f492dc_170.jpg".to_owned()
	}
	fn author_url(&self) -> String {
		format!("https://www.pixiv.net/en/users/{}", &self.author_username())
	}

	fn media(&self) -> Vec<String> {
		vec![format!("https://embed.pixiv.net/decorate.php?illust_id={}", &self.id)]
	}
}

pub struct PixivAgent {
	link: AgentLink<Self>,
	subscribers: HashSet<HandlerId>,
	endpoints: Vec<Rc<dyn Endpoint>>,
}

pub enum AgentRequest {
	//UpdateRateLimit(RateLimit),
	EventBusMsg(String),
}

pub enum AgentOutput {
	//UpdatedRateLimit(RateLimit),
}

pub enum AgentMsg {
	Init,
}

impl Agent for PixivAgent {
	type Reach = Context<Self>;
	type Message = AgentMsg;
	type Input = AgentRequest;
	type Output = String;

	fn create(link: AgentLink<Self>) -> Self {
		link.send_message(AgentMsg::Init);

		Self {
			link,
			subscribers: HashSet::new(),
			endpoints: vec![Rc::from(PixivEndpoint {
				article: Rc::from(PixivArticleData {
					id: "92885703".to_owned()
				})
			})]
		}
	}

	fn update(&mut self, msg: Self::Message) {
		match msg {
			AgentMsg::Init => {
				EndpointAgent::dispatcher().send(EndpointRequest::AddEndpoint(self.endpoints[0].clone()));
			}
		}
	}

	fn connected(&mut self, id: HandlerId) {
		self.subscribers.insert(id);
	}

	fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
		match msg {
			AgentRequest::EventBusMsg(s) => {
				for sub in self.subscribers.iter() {
					self.link.respond(*sub, s.clone());
				}
			}
		}
	}

	fn disconnected(&mut self, id: HandlerId) {
		self.subscribers.remove(&id);
	}
}

pub struct PixivEndpoint {
	article: Rc<dyn SocialArticleData>
}

impl Endpoint for PixivEndpoint {
	fn name(&self) -> String {
		"Hard-coded Pixiv Endpoint".to_owned()
	}

	fn refresh(&self) {}
}