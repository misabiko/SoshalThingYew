use yew::prelude::*;
use yew_agent::{Agent, AgentLink, HandlerId, Context};
use std::rc::Weak;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use crate::articles::ArticleData;

pub struct ServiceActions {
	pub like: Callback<(HandlerId, Weak<RefCell<dyn ArticleData>>)>,
	pub repost: Callback<(HandlerId, Weak<RefCell<dyn ArticleData>>)>,
	pub mark_as_read: Callback<(HandlerId, Weak<RefCell<dyn ArticleData>>, bool)>,
}

pub struct ArticleActionsAgent {
	link: AgentLink<Self>,
	services: HashMap<&'static str, ServiceActions>,
	subscribers: HashSet<HandlerId>,
}

pub enum Request {
	Init(&'static str, ServiceActions),
	Callback(Vec<Weak<RefCell<dyn ArticleData>>>),
	Like(Weak<RefCell<dyn ArticleData>>),
	Repost(Weak<RefCell<dyn ArticleData>>),
	MarkAsRead(Weak<RefCell<dyn ArticleData>>, bool),
}

pub enum Response {
	Callback(Vec<Weak<RefCell<dyn ArticleData>>>),
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
			Request::Callback(articles) => {
				for sub in &self.subscribers {
					if sub.is_respondable() {
						self.link.respond(*sub, Response::Callback(articles.clone()));
					}
				}
			},
			Request::Like(article) => {
				let strong = article.upgrade().unwrap();
				let borrow = strong.borrow();

				self.services.get(&borrow.service()).map(|s| s.like.emit((id, article.clone())));
			}
			Request::Repost(article) => {
				let strong = article.upgrade().unwrap();
				let borrow = strong.borrow();

				self.services.get(&borrow.service()).map(|s| s.repost.emit((id, article.clone())));
			}
			Request::MarkAsRead(article, value) => {
				let strong = article.upgrade().unwrap();
				let borrow = strong.borrow();

				self.services.get(&borrow.service()).map(|s| s.mark_as_read.emit((id, article.clone(), value)));
			}
		};
	}

	fn disconnected(&mut self, id: HandlerId) {
		self.subscribers.remove(&id);
	}
}