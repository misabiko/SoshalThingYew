use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use js_sys::Date;
use yew_agent::{Agent, AgentLink, Context, Dispatcher, Dispatched, HandlerId};

use crate::articles::{ArticleBox, ArticleData, ArticleMedia, ArticleRc, ArticleWeak};
use crate::{Endpoint, EndpointId, EndpointRequest};
use crate::services::{
	service,
	EndpointSerialized,
	RefreshTime,
	endpoint_agent::{EndpointAgent, EndpointConstructor, EndpointConstructorCollection},
	article_actions::{ArticleActionsAgent, ServiceActions, ArticleActionsRequest},
};

#[service("Dummy", DummyArticleData, u32)]
pub struct DummyServiceAgent {
	//link: AgentLink<Self>,
	_endpoint_agent: Dispatcher<EndpointAgent>,
	actions_agent: Dispatcher<ArticleActionsAgent>,
}

pub enum DummyServiceMsg {
	Like(HandlerId, ArticleWeak),
	Repost(HandlerId, ArticleWeak),
}

type Msg = DummyServiceMsg;

impl Agent for DummyServiceAgent {
	type Reach = Context<Self>;
	type Message = Msg;
	type Input = ();
	type Output = ();

	fn create(link: AgentLink<Self>) -> Self {
		let articles = HashMap::from([
			(0, Rc::new(RefCell::new(DummyArticleData {
				id: 0,
				creation_time: js_sys::Date::new_0(),
				text: "Text".to_string(),
				author_name: "Author Name".to_string(),
				author_avatar_url: "".to_string(),
				author_url: "".to_string(),
				url: "".to_string(),
				marked_as_read: false,
				hidden: false,
				liked: false,
				reposted: false,
			})))
		]);

		let weak_articles: Vec<ArticleWeak> = articles.values()
			.map(|a| Rc::downgrade(a) as ArticleWeak)
			.collect();

		let mut _endpoint_agent = EndpointAgent::dispatcher();
		_endpoint_agent.send(EndpointRequest::InitService(
			SERVICE_INFO.name,
			EndpointConstructorCollection {
				constructors: vec![
					EndpointConstructor {
						name: "Endpoint",
						param_template: vec![],
						callback: Rc::new(move |id, _params| Box::new(DummyEndpoint::new(
							id,
							weak_articles.clone(),
						))),
					}
				],
				user_endpoint_index: None,
			},
		));

		let mut actions_agent = ArticleActionsAgent::dispatcher();
		actions_agent.send(ArticleActionsRequest::Init(SERVICE_INFO.name, ServiceActions {
			like: Some(link.callback(|(id, article)| Msg::Like(id, article))),
			repost: Some(link.callback(|(id, article)| Msg::Repost(id, article))),
			fetch_data: None,
		}));

		Self {
			//link,
			_endpoint_agent,
			actions_agent,
			articles,
		}
	}

	fn update(&mut self, msg: Self::Message) {
		match msg {
			Msg::Like(_id, article) => {
				let strong = article.upgrade().unwrap();

				let article = self.articles.get(&strong.borrow().id().parse::<u32>().unwrap()).unwrap();
				let old_liked = strong.borrow().liked();
				article.borrow_mut().liked = !old_liked;

				self.actions_agent.send(ArticleActionsRequest::RedrawTimelines(vec![Rc::downgrade(article) as ArticleWeak]));
			}
			Msg::Repost(_id, article) => {
				let strong = article.upgrade().unwrap();

				let article = self.articles.get(&strong.borrow().id().parse::<u32>().unwrap()).unwrap();
				let old_reposted = strong.borrow().reposted();
				article.borrow_mut().reposted = !old_reposted;

				self.actions_agent.send(ArticleActionsRequest::RedrawTimelines(vec![Rc::downgrade(article) as ArticleWeak]));
			}
		}
	}

	fn handle_input(&mut self, _msg: Self::Input, _id: HandlerId) {}
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
	liked: bool,
	reposted: bool,
}

impl ArticleData for DummyArticleData {
	//type Id = u32;

	fn service(&self) -> &'static str { SERVICE_INFO.name }

	fn id(&self) -> String { self.id.to_string() }

	fn sortable_id(&self) -> u64 { self.id as u64 }

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

	fn liked(&self) -> bool {
		self.liked
	}

	fn like_count(&self) -> u32 {
		if self.liked { 1 } else { 0 }
	}

	fn reposted(&self) -> bool {
		self.reposted
	}

	fn repost_count(&self) -> u32 {
		if self.reposted { 1 } else { 0 }
	}

	fn clone_data(&self) -> ArticleBox {
		Box::new(self.clone())
	}

	fn media_loaded(&mut self, _index: usize) {
		log::warn!("Dummy Service doesn't do lazy loading.");
	}
}

pub struct DummyEndpoint {
	id: EndpointId,
	articles: Vec<ArticleWeak>,
	//agent: Dispatcher<DummyServiceAgent>,
	endpoint_agent: Dispatcher<EndpointAgent>,
}

impl DummyEndpoint {
	pub fn new(id: EndpointId, articles: Vec<ArticleWeak>) -> Self {
		Self {
			id,
			articles,
			//agent: DummyServiceAgent::dispatcher(),
			endpoint_agent: EndpointAgent::dispatcher(),
		}
	}
}

impl Endpoint for DummyEndpoint {
	fn name(&self) -> String {
		"Dummy Endpoint".to_owned()
	}

	fn id(&self) -> &EndpointId {
		&self.id
	}

	fn articles(&mut self) -> &mut Vec<ArticleWeak> {
		&mut self.articles
	}

	fn refresh(&mut self, refresh_time: RefreshTime) {
		self.endpoint_agent.send(EndpointRequest::AddArticles(
			refresh_time,
			self.id,
			self.articles.iter()
				.map(|a| a.upgrade().unwrap())
				.collect(),
		));
	}

	fn eq_storage(&self, storage: &EndpointSerialized) -> bool {
		storage.service == SERVICE_INFO.name &&
			storage.endpoint_type == 0
	}
}