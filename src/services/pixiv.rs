use std::rc::{Rc, Weak};
use std::cell::{RefCell, Ref};
use yew_agent::{Agent, AgentLink, Context, HandlerId, Dispatched, Dispatcher};
use js_sys::Date;
use std::collections::HashMap;

use crate::articles::{ArticleData, ArticleMedia};
use crate::services::Endpoint;
use crate::services::endpoint_agent::{EndpointAgent, Request as EndpointRequest, EndpointId, RefreshTime, EndpointConstructors};

pub struct PixivArticleData {
	id: u32,
	src: String,
	title: String,
	author_name: String,
	author_id: u32,
	author_avatar_url: String,
	marked_as_read: bool,
	hidden: bool,
}

impl ArticleData for PixivArticleData {
	fn service(&self) -> &'static str {
		"Pixiv"
	}
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

	fn media(&self) -> Vec<ArticleMedia> {
		//TODO Pixiv image ratio
		vec![ArticleMedia::Image(self.src.clone(), 1.0)]
	}

	fn url(&self) -> String {
		format!("https://www.pixiv.net/en/artworks/{}", &self.id)
	}

	fn update(&mut self, new: &Ref<dyn ArticleData>) {
		self.src = match new.media().first() {
			Some(ArticleMedia::Image(src, _ratio)) => src.clone(),
			_ => "".to_owned(),
		};
		self.title = new.text();
	}
	fn marked_as_read(&self) -> bool {
		self.marked_as_read.clone()
	}
	fn set_marked_as_read(&mut self, value: bool) {
		self.marked_as_read = value;
	}
	fn hidden(&self) -> bool {
		self.hidden.clone()
	}
	fn set_hidden(&mut self, value: bool) {
		self.hidden = value;
	}
}

pub struct PixivAgent {
	endpoint_agent: Dispatcher<EndpointAgent>,
	articles: HashMap<u32, Rc<RefCell<PixivArticleData>>>,
}

pub enum Request {
	AddArticles(RefreshTime, EndpointId, Vec<Rc<RefCell<PixivArticleData>>>),
}

impl Agent for PixivAgent {
	type Reach = Context<Self>;
	type Message = ();
	type Input = Request;
	type Output = ();

	fn create(_link: AgentLink<Self>) -> Self {
		let mut endpoint_agent = EndpointAgent::dispatcher();
		endpoint_agent.send(EndpointRequest::InitService(
			"Pixiv".to_owned(),
			EndpointConstructors {
				endpoint_types: vec![],
				user_endpoint: None,
			}));

		Self {
			endpoint_agent,
			articles: HashMap::new(),
		}
	}

	fn update(&mut self, _msg: Self::Message) {}

	fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
		match msg {
			Request::AddArticles(refresh_time, endpoint_id, articles) => {
				let mut valid_rc = Vec::new();
				for article in articles.into_iter() {
					let borrow = article.borrow();
					let valid_a_rc = self.articles.entry(borrow.id)
						.and_modify(|a| a.borrow_mut().update(&(borrow as Ref<dyn ArticleData>)))
						.or_insert_with(|| article.clone()).clone();

					valid_rc.push(valid_a_rc);
				}
				self.endpoint_agent.send(EndpointRequest::AddArticles(
					refresh_time,
					endpoint_id,
					valid_rc.into_iter()
						.map(|article| article as Rc<RefCell<dyn ArticleData>>)
						.collect(),
				))
			},
		};
	}
}

pub struct FollowEndpoint {
	id: EndpointId,
	articles: Vec<Weak<RefCell<dyn ArticleData>>>,
	agent: Dispatcher<PixivAgent>,
}

impl FollowEndpoint {
	pub fn new(id: EndpointId) -> Self {
		Self {
			id,
			articles: Vec::new(),
			agent: PixivAgent::dispatcher(),
		}
	}
}

fn parse_article(element: web_sys::Element) -> Option<Rc<RefCell<PixivArticleData>>> {
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

	Some(Rc::new(RefCell::new(PixivArticleData {
		id,
		src,
		author_avatar_url,
		title,
		author_id,
		author_name,
		marked_as_read: false,
		hidden: false,
	})))
}

impl Endpoint for FollowEndpoint {
	fn name(&self) -> String {
		"Follow Endpoint".to_owned()
	}

	fn id(&self) -> &EndpointId {
		&self.id
	}

	fn articles(&mut self) -> &mut Vec<Weak<RefCell<dyn ArticleData>>> {
		&mut self.articles
	}

	fn refresh(&mut self, refresh_time: RefreshTime) {
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
		self.agent.send(Request::AddArticles(refresh_time, id, articles));
	}
}