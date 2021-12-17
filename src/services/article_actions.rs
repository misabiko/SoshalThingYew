use yew::prelude::*;
use yew_agent::{Agent, AgentLink, HandlerId, Context};
use std::rc::Weak;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::articles::ArticleData;

pub struct ServiceActions {
	pub like: Callback<Weak<RefCell<dyn ArticleData>>>,
	pub repost: Callback<Weak<RefCell<dyn ArticleData>>>,
}

pub struct ArticleActionsAgent {
	services: HashMap<&'static str, ServiceActions>
}

pub enum Msg {}

pub enum Request {
	Init(&'static str, ServiceActions),
	Like(Weak<RefCell<dyn ArticleData>>),
	Repost(Weak<RefCell<dyn ArticleData>>),
}

impl Agent for ArticleActionsAgent {
	type Reach = Context<Self>;
	type Message = Msg;
	type Input = Request;
	type Output = ();

	fn create(_link: AgentLink<Self>) -> Self {
		log::debug!("New agent?");
		Self {
			services: HashMap::new(),
		}
	}

	fn update(&mut self, msg: Self::Message) {
		match msg {

		}
	}

	fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
		match msg {
			Request::Init(service, actions) => {
				self.services.insert(service, actions);
			}
			Request::Like(article) => {
				let strong = article.upgrade().unwrap();
				let borrow = strong.borrow();

				self.services.get(&borrow.service()).map(|s| s.like.emit(article.clone()));
			}
			Request::Repost(article) => {
				let strong = article.upgrade().unwrap();
				let borrow = strong.borrow();

				self.services.get(&borrow.service()).map(|s| s.repost.emit(article.clone()));
			}
		};
	}
}