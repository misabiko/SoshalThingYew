use yew::prelude::*;
use yew_agent::{Agent, AgentLink, HandlerId, Context};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

use crate::articles::{ArticleWeak, weak_actual_article};
use crate::services::storages::{hide_article, mark_article_as_read};

pub struct ServiceActions {
	pub like: Option<Callback<(HandlerId, ArticleWeak)>>,
	pub repost: Option<Callback<(HandlerId, ArticleWeak)>>,
	pub fetch_data: Option<Callback<(HandlerId, ArticleWeak)>>,
}

pub struct ArticleActionsAgent {
	link: AgentLink<Self>,
	services: HashMap<&'static str, ServiceActions>,
	subscribers: HashSet<HandlerId>,
}

pub enum Request {
	Init(&'static str, ServiceActions),
	//Callback(Vec<ArticleWeak>),
	Action(Action, Vec<ArticleWeak>),
	RedrawTimelines(Vec<ArticleWeak>),
}

pub enum Response {
	//Callback(Vec<ArticleWeak>),
	RedrawTimelines(Vec<ArticleWeak>),
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
			Request::RedrawTimelines(articles) => self.redraw_timelines(articles),
			Request::Action(action, articles) => {
				match action {
					Action::Like => {
						for article in &articles {
							let strong = article.upgrade().unwrap();
							let borrow = strong.borrow();

							self.services.get(&borrow.service())
								.and_then(|s| s.like.as_ref())
								.map(|l| l.emit((id, article.clone())));
						}
					}
					Action::Repost => {
						for article in &articles {
							let strong = article.upgrade().unwrap();
							let borrow = strong.borrow();

							self.services.get(&borrow.service())
								.and_then(|s| s.repost.as_ref())
								.map(|r| r.emit((id, article.clone())));
						}
					}
					Action::MarkAsRead => {
						for article in &articles {
							let strong = weak_actual_article(&article).upgrade().unwrap();
							let mut borrow = strong.borrow_mut();

							let new_marked_as_read = !borrow.marked_as_read();
							borrow.set_marked_as_read(new_marked_as_read);

							mark_article_as_read(borrow.service(), borrow.id(), new_marked_as_read);
						}
					}
					Action::Hide => {
						for article in &articles {
							let strong = weak_actual_article(&article).upgrade().unwrap();
							let mut borrow = strong.borrow_mut();

							let new_hidden = !borrow.hidden();
							borrow.set_hidden(new_hidden);

							hide_article(borrow.service(), borrow.id(), new_hidden);
						}
					}
					Action::FetchData => {
						for article in &articles {
							let strong = article.upgrade().unwrap();
							let borrow = strong.borrow();

							self.services.get(&borrow.service())
								.and_then(|s| s.fetch_data.as_ref())
								.map(|f| f.emit((id, article.clone())));
						}
					}
				};

				self.redraw_timelines(articles);
			}
		};
	}

	fn disconnected(&mut self, id: HandlerId) {
		self.subscribers.remove(&id);
	}
}

impl ArticleActionsAgent {
	fn redraw_timelines(&self, articles: Vec<ArticleWeak>) {
		for sub in &self.subscribers {
			if sub.is_respondable() {
				self.link.respond(*sub, Response::RedrawTimelines(articles.clone()));
			}
		}
	}
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Action {
	Like,
	Repost,
	MarkAsRead,
	Hide,
	FetchData,
}

impl Display for Action {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Action::Like => write!(f, "Like"),
			Action::Repost => write!(f, "Repost"),
			Action::MarkAsRead => write!(f, "Mark As Read"),
			Action::Hide => write!(f, "Hide"),
			Action::FetchData => write!(f, "Fetch Data"),
		}
	}
}