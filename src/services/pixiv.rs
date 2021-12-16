use std::rc::Rc;
use yew_agent::{Agent, AgentLink, Context, HandlerId, Dispatched, Dispatcher};
use js_sys::Date;

use crate::articles::ArticleData;
use crate::services::endpoints::{EndpointAgent, Endpoint, Request as EndpointRequest, EndpointId};

pub struct PixivArticleData {
	id: u32,
	src: String,
	title: String,
	author_name: String,
	author_id: u32,
	author_avatar_url: String,
}

impl ArticleData for PixivArticleData {
	fn id(&self) -> String {
		self.id.clone().to_string()
	}
	fn creation_time(&self) -> Date {
		js_sys::Date::new_0()
	}
	fn text(&self) -> String {
		self.title.clone()
	}
	fn author_username(&self) -> String {
		self.author_id.clone().to_string()
	}
	fn author_name(&self) -> String {
		self.author_name.clone()
	}
	fn author_avatar_url(&self) -> String {
		self.author_avatar_url.clone()
	}
	fn author_url(&self) -> String {
		format!("https://www.pixiv.net/en/users/{}", &self.author_id)
	}

	fn media(&self) -> Vec<String> {
		vec![self.src.clone()]
	}
}

pub struct PixivAgent;

pub enum Msg {
	Init,
}

impl Agent for PixivAgent {
	type Reach = Context<Self>;
	type Message = Msg;
	type Input = ();
	type Output = ();

	fn create(_link: AgentLink<Self>) -> Self {
		Self {}
	}

	fn update(&mut self, _msg: Self::Message) {}

	fn handle_input(&mut self, _msg: Self::Input, _id: HandlerId) {}
}

pub struct FollowEndpoint {
	id: EndpointId,
	articles: Vec<Rc<dyn ArticleData>>,
	endpoint_agent: Dispatcher<EndpointAgent>,
}

impl FollowEndpoint {
	pub fn new(id: EndpointId) -> Self {
		Self {
			id,
			articles: Vec::new(),
			endpoint_agent: EndpointAgent::dispatcher(),
		}
	}
}

fn parse_article(element: web_sys::Element) -> Option<Rc<dyn ArticleData>> {
	let anchors = element.get_elements_by_tag_name("a");
	let id = match anchors.get_with_index(0) {
		Some(a) => match a.get_attribute("data-gtm-value") {
			Some(id) => match id.parse::<u32>() {
				Ok(id) => id,
				Err(_) => return None,
			},
			None => return None
		},
		None => return None,
	};
	let title = match anchors.get_with_index(1) {
		Some(a) => match a.text_content() {
			Some(title) => title,
			None => return None
		},
		None => return None,
	};
	let (author_id, author_name) = match anchors.get_with_index(3) {
		Some(a) => ( match a.get_attribute("data-gtm-value") {
			Some(id) => match id.parse::<u32>() {
				Ok(id) => id,
				Err(_) => return None,
			},
			None => return None
		}, match a.text_content() {
				Some(title) => title,
				None => return None
		}),
		None => return None,
	};

	let imgs = element.get_elements_by_tag_name("img");
	let src = match imgs.get_with_index(0) {
		Some(img) => match img.get_attribute("src") {
			Some(src) => src,
			None => return None,
		}
		None => return None,
	};

	let author_avatar_url = match imgs.get_with_index(1) {
		Some(img) => match img.get_attribute("src") {
			Some(src) => src,
			None => return None,
		}
		None => return None,
	};

	Some(Rc::new(PixivArticleData {
		id,
		src,
		author_avatar_url,
		title,
		author_id,
		author_name
	}))
}

impl Endpoint for FollowEndpoint {
	fn name(&self) -> String {
		"Follow Endpoint".to_owned()
	}

	fn id(&self) -> &EndpointId {
		&self.id
	}

	fn articles(&mut self) -> &mut Vec<Rc<dyn ArticleData>> {
		&mut self.articles
	}

	fn refresh(&mut self) {
		let mut articles = Vec::new();
		let posts_selector = gloo_utils::document()
			.query_selector(".sc-9y4be5-1.jtUPOE");
		if let Ok(Some(posts)) = posts_selector {
			let children = posts.children();
			for i in 0..children.length() {
				if let Some(article) = children.get_with_index(i).and_then(parse_article) {
					articles.push(article);
				}
			}
		}

		let id = self.id().clone();
		self.endpoint_agent.send(EndpointRequest::AddArticles(id, articles));
	}
}