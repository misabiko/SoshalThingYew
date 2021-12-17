use yew::prelude::*;
use yew_agent::{Agent, AgentLink, HandlerId, Context};
use std::rc::Weak;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::articles::ArticleData;

pub struct ServiceActions {
	pub like: Callback<(HandlerId, Weak<RefCell<dyn ArticleData>>)>,
	pub repost: Callback<(HandlerId, Weak<RefCell<dyn ArticleData>>)>,
}

pub struct ArticleActionsAgent {
	link: AgentLink<Self>,
	services: HashMap<&'static str, ServiceActions>
}

pub enum Request {
	Init(&'static str, ServiceActions),
	Callback(HandlerId),
	Like(Weak<RefCell<dyn ArticleData>>),
	Repost(Weak<RefCell<dyn ArticleData>>),
}

pub enum Response {
	Callback,
}

impl Agent for ArticleActionsAgent {
	type Reach = Context<Self>;
	type Message = ();
	type Input = Request;
	type Output = Response;

	fn create(link: AgentLink<Self>) -> Self {
		Self {
			link,
			services: HashMap::new(),
		}
	}

	fn update(&mut self, _msg: Self::Message) {}

	fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
		match msg {
			Request::Init(service, actions) => {
				self.services.insert(service, actions);
			}
			Request::Callback(respond_id) => self.link.respond(respond_id, Response::Callback),
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
		};
	}
}