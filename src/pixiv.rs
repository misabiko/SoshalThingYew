use std::rc::Rc;
use yew_agent::{Agent, AgentLink, Context, HandlerId, Dispatched, Dispatcher};

use crate::articles::SocialArticleData;
use crate::endpoints::{EndpointAgent, Endpoint, EndpointRequest, EndpointId};

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
	endpoint_agent: Dispatcher<EndpointAgent>,
}

pub enum AgentRequest {
	//UpdateRateLimit(RateLimit),
	AddArticle(EndpointId, Rc<PixivArticleData>)
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
			endpoint_agent: EndpointAgent::dispatcher(),
		}
	}

	fn update(&mut self, msg: Self::Message) {
		match msg {
			AgentMsg::Init => {
				EndpointAgent::dispatcher().send(EndpointRequest::AddEndpoint(Box::new(|id|
					Box::new(PixivEndpoint {
						id,
						article: Rc::from(PixivArticleData {
							id: "92885703".to_owned()
						})
					})
				)));
			}
		}
	}

	fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
		match msg {
			AgentRequest::AddArticle(id, a) => self.endpoint_agent.send(EndpointRequest::AddArticle(id, a))
		}
	}
}

pub struct PixivEndpoint {
	id: EndpointId,
	article: Rc<dyn SocialArticleData>,
}

impl Endpoint for PixivEndpoint {
	fn name(&self) -> String {
		"Hard-coded Pixiv Endpoint".to_owned()
	}

	fn id(&self) -> &EndpointId {
		&self.id
	}

	fn refresh(&mut self) {
		//self.agent.
	}
}