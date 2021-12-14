use std::rc::Rc;
use yew_agent::{Agent, AgentLink, Context, HandlerId, Dispatched, Dispatcher};
use js_sys::Date;

use crate::articles::SocialArticleData;
use crate::endpoints::{EndpointAgent, Endpoint, Request as EndpointRequest, EndpointId};

pub struct PixivArticleData {
	id: String,
	creation_time: Date,
}

impl SocialArticleData for PixivArticleData {
	fn id(&self) -> String {
		self.id.clone()
	}
	fn creation_time(&self) -> Date {
		self.creation_time.clone()
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
	//endpoint_agent: Dispatcher<EndpointAgent>,
}

pub enum Msg {
	Init,
}

impl Agent for PixivAgent {
	type Reach = Context<Self>;
	type Message = Msg;
	type Input = ();
	type Output = ();

	fn create(link: AgentLink<Self>) -> Self {
		link.send_message(Msg::Init);

		Self {
			//endpoint_agent: EndpointAgent::dispatcher(),
		}
	}

	fn update(&mut self, msg: Self::Message) {
		match msg {
			Msg::Init => {
				EndpointAgent::dispatcher().send(EndpointRequest::AddEndpoint(Box::new(|id|
					Box::new(PixivEndpoint::new(id))
				)));
			}
		}
	}

	fn handle_input(&mut self, _msg: Self::Input, _id: HandlerId) {}
}

pub struct PixivEndpoint {
	id: EndpointId,
	article: Rc<dyn SocialArticleData>,
	endpoint_agent: Dispatcher<EndpointAgent>,
}

impl PixivEndpoint {
	pub fn new(id: EndpointId) -> Self {
		Self {
			id,
			article: Rc::from(PixivArticleData {
				id: "92885703".to_owned(),
				creation_time: Date::new_0(),
			}),
			endpoint_agent: EndpointAgent::dispatcher(),
		}
	}
}

impl Endpoint for PixivEndpoint {
	fn name(&self) -> String {
		"Hard-coded Pixiv Endpoint".to_owned()
	}

	fn id(&self) -> &EndpointId {
		&self.id
	}

	fn refresh(&mut self) {
		let id = self.id().clone();
		self.endpoint_agent.send(EndpointRequest::AddArticles(id, vec![self.article.clone(); 10]));
	}
}