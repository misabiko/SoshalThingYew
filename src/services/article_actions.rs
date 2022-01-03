use yew::prelude::*;
use yew_agent::{Agent, AgentLink, HandlerId, Context};
use std::rc::Weak;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use crate::articles::ArticleData;

pub struct ServiceActions {
	pub like: Option<Callback<(HandlerId, Weak<RefCell<dyn ArticleData>>)>>,
	pub repost: Option<Callback<(HandlerId, Weak<RefCell<dyn ArticleData>>)>>,
	pub fetch_data: Option<Callback<(HandlerId, Weak<RefCell<dyn ArticleData>>)>>,
}

pub struct ArticleActionsAgent {
	link: AgentLink<Self>,
	services: HashMap<&'static str, ServiceActions>,
	subscribers: HashSet<HandlerId>,
}

pub enum Request {
	Init(&'static str, ServiceActions),
	//Callback(Vec<Weak<RefCell<dyn ArticleData>>>),
	Like(Weak<RefCell<dyn ArticleData>>),
	Repost(Weak<RefCell<dyn ArticleData>>),
	FetchData(Weak<RefCell<dyn ArticleData>>),
	RedrawTimelines(Vec<Weak<RefCell<dyn ArticleData>>>),
}

pub enum Response {
	//Callback(Vec<Weak<RefCell<dyn ArticleData>>>),
	RedrawTimelines(Vec<Weak<RefCell<dyn ArticleData>>>),
}

impl Agent for ArticleActionsAgent {
	type Reach = Context<Self>;
	type Message = ();
	type Input = Request;
	type Output = Response;

	fn create(link: AgentLink<Self>) -> Self {
		Self {
			link,
			subscribers: HashSet::new(),
			services: HashMap::new(),
		}
	}

	fn update(&mut self, _msg: Self::Message) {}

	fn connected(&mut self, id: HandlerId) {
		self.subscribers.insert(id);
	}

	fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
		match msg {
			Request::Init(service, actions) => {
				self.services.insert(service, actions);
			}
			Request::RedrawTimelines(articles) => {
				for sub in &self.subscribers {
					if sub.is_respondable() {
						self.link.respond(*sub, Response::RedrawTimelines(articles.clone()));
					}
				}
			}
			/*Request::Callback(articles) => {
				for sub in &self.subscribers {
					if sub.is_respondable() {
						self.link.respond(*sub, Response::Callback(articles.clone()));
					}
				}
			},*/
			Request::Like(article) => {
				let strong = article.upgrade().unwrap();
				let borrow = strong.borrow();

				self.services.get(&borrow.service())
					.and_then(|s| s.like.as_ref())
					.map(|l| l.emit((id, article.clone())));
			}
			Request::Repost(article) => {
				let strong = article.upgrade().unwrap();
				let borrow = strong.borrow();

				self.services.get(&borrow.service())
					.and_then(|s| s.repost.as_ref())
					.map(|r| r.emit((id, article.clone())));
			}
			Request::FetchData(article) => {
				let strong = article.upgrade().unwrap();
				let borrow = strong.borrow();

				self.services.get(&borrow.service())
					.and_then(|s| s.fetch_data.as_ref())
					.map(|f| f.emit((id, article.clone())));
			}
		};
	}

	fn disconnected(&mut self, id: HandlerId) {
		self.subscribers.remove(&id);
	}
}