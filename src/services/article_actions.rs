use yew::prelude::*;
use yew_agent::{Agent, AgentLink, HandlerId, Context};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use web_sys::console;
use wasm_bindgen::JsValue;

use crate::articles::ArticleWeak;
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
				for article in &articles {
					let strong = article.upgrade().unwrap();
					let mut borrow = strong.borrow_mut();

					match action {
						Action::Like => {
							self.services.get(&borrow.service())
								.and_then(|s| s.like.as_ref())
								.map(|l| l.emit((id, article.clone())));
						}
						Action::Repost => {
							self.services.get(&borrow.service())
								.and_then(|s| s.repost.as_ref())
								.map(|r| r.emit((id, article.clone())));
						}
						Action::MarkAsRead => {
							let new_marked_as_read = !borrow.marked_as_read();
							borrow.set_marked_as_read(new_marked_as_read);

							mark_article_as_read(borrow.service(), borrow.id(), new_marked_as_read);
						}
						Action::Hide => {
							let new_hidden = !borrow.hidden();
							borrow.set_hidden(new_hidden);

							hide_article(borrow.service(), borrow.id(), new_hidden);
						}
						Action::FetchData => {
							self.services.get(&borrow.service())
								.and_then(|s| s.fetch_data.as_ref())
								.map(|f| f.emit((id, article.clone())));
						}
						Action::LogData => {
							log::info!("{:#?}", &borrow);
						}
						Action::LogJsonData => {
							let json = &borrow.json();
							let is_mobile = web_sys::window().expect("couldn't get global window")
								.navigator().user_agent()
								.map(|n| n.contains("Mobile"))
								.unwrap_or(false);
							if is_mobile {
								log::info!("{}", serde_json::to_string_pretty(json).unwrap_or("Couldn't parse json data.".to_owned()));
							}else {
								console::dir_1(&JsValue::from_serde(&json).unwrap_or_default());
							}
						}
					};
				}

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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Action {
	Like,
	Repost,
	MarkAsRead,
	Hide,
	FetchData,
	LogData,
	LogJsonData,
}

impl Display for Action {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Action::MarkAsRead => write!(f, "Mark As Read"),
			Action::FetchData => write!(f, "Fetch Data"),
			_ => f.write_fmt(format_args!("{:?}", self)),
		}
	}
}

const ALL_ACTIONS: [Action; 7] = [
	Action::Like,
	Action::Repost,
	Action::MarkAsRead,
	Action::Hide,
	Action::FetchData,
	Action::LogData,
	Action::LogJsonData,
];

impl Action {
	pub fn iter() -> impl ExactSizeIterator<Item=&'static Action> {
		ALL_ACTIONS.iter()
	}
}