use std::rc::{Rc, Weak};
use std::cell::{RefCell, Ref};
use yew_agent::{Agent, AgentLink, Context, HandlerId, Dispatched, Dispatcher, Bridge};
use yew_agent::utils::store::{StoreWrapper, ReadOnly, Bridgeable};
use js_sys::Date;

use crate::articles::{ArticleData, ArticleMedia};
use crate::services::endpoints::{EndpointStore, Endpoint, Request as EndpointRequest, EndpointId, RefreshTime};

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
		vec![ArticleMedia::Image(self.src.clone())]
	}

	fn url(&self) -> String {
		format!("https://www.pixiv.net/en/artworks/{}", &self.id)
	}

	fn update(&mut self, new: &Ref<dyn ArticleData>) {
		self.src = match new.media().first() {
			Some(ArticleMedia::Image(src)) => src.clone(),
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
	endpoint_store: Box<dyn Bridge<StoreWrapper<EndpointStore>>>,
}

pub enum Msg {
	EndpointStoreResponse(ReadOnly<EndpointStore>),
}

pub enum Request {
	AddArticles(RefreshTime, EndpointId, Vec<Rc<RefCell<dyn ArticleData>>>),
}

impl Agent for PixivAgent {
	type Reach = Context<Self>;
	type Message = Msg;
	type Input = Request;
	type Output = ();

	fn create(link: AgentLink<Self>) -> Self {
		let mut endpoint_store = EndpointStore::bridge(link.callback(Msg::EndpointStoreResponse));
		endpoint_store.send(EndpointRequest::InitService("Pixiv".to_owned(), vec![

		]));

		Self {
			endpoint_store,
		}
	}

	fn update(&mut self, msg: Self::Message) {
		match msg {
			Msg::EndpointStoreResponse(_) => {}
		}
	}

	fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
		match msg {
			Request::AddArticles(refresh_time, endpoint_id, articles) =>
				self.endpoint_store.send(EndpointRequest::AddArticles(
					refresh_time,
					endpoint_id,
					articles
				)),
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

fn parse_article(element: web_sys::Element) -> Option<Rc<RefCell<dyn ArticleData>>> {
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