use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use reqwest::Url;
use yew_agent::{Agent, AgentLink, Context, Dispatcher, Dispatched, HandlerId};
use yew::prelude::*;

mod article;
mod endpoints;

use article::YouTubeArticleData;
use crate::articles::ArticleData;
use crate::error::{Result, Error};
use crate::notifications::{Notification, NotificationAgent, Request as NotificationRequest};
use crate::services::endpoint_agent::{EndpointAgent, EndpointConstructor, EndpointId, Request as EndpointRequest};
use crate::services::article_actions::{ArticleActionsAgent, ServiceActions, Request as ArticleActionsRequest};
use crate::services::endpoint_agent::EndpointConstructors;
use crate::services::RefreshTime;
use crate::services::storages::get_service_storage;
use crate::services::youtube::endpoints::{fetch_videos, PlaylistEndpoint};

#[derive(Debug)]
enum AuthState {
	NotLoggedIn,
	LoggedIn,
}

//TODO derive(Service)?
//TODO type<A> ServiceArticles = HashMap<A::Id, Rc<RefCell<A>>>
pub struct YouTubeAgent {
	link: AgentLink<Self>,
	endpoint_agent: Dispatcher<EndpointAgent>,
	actions_agent: Dispatcher<ArticleActionsAgent>,
	articles: HashMap<String, Rc<RefCell<YouTubeArticleData>>>,
	auth_state: AuthState,
	sidebar_handler: Option<HandlerId>,
	notification_agent: Dispatcher<NotificationAgent>,
}

pub enum Msg {
	EndpointFetchResponse(RefreshTime, EndpointId, Result<Vec<Rc<RefCell<YouTubeArticleData>>>>),
}

pub enum Request {
	Auth(bool),
	AddArticles(RefreshTime, EndpointId, Vec<Rc<RefCell<YouTubeArticleData>>>),
	FetchArticles(RefreshTime, EndpointId, Url),
	Sidebar,
}

pub enum Response {
	Sidebar(Html),
}

impl Agent for YouTubeAgent {
	type Reach = Context<Self>;
	type Message = Msg;
	type Input = Request;
	type Output = Response;

	fn create(link: AgentLink<Self>) -> Self {
		let mut endpoint_agent = EndpointAgent::dispatcher();
		endpoint_agent.send(EndpointRequest::InitService(
			"YouTube".to_owned(),
			EndpointConstructors {
				endpoint_types: vec![
					EndpointConstructor {
						name: "Playlist",
						param_template: vec![
							("id", serde_json::Value::String("".to_owned()))
						],
						callback: Rc::new(|id, params| Box::new(PlaylistEndpoint::from_json(id, params))),
					},
				],
				user_endpoint: None,
			}));

		let mut actions_agent = ArticleActionsAgent::dispatcher();
		actions_agent.send(ArticleActionsRequest::Init("YouTube", ServiceActions {
			like: None,
			repost: None,
			fetch_data: None,
		}));

		Self {
			endpoint_agent,
			link,
			actions_agent,
			articles: HashMap::new(),
			auth_state: AuthState::NotLoggedIn,
			sidebar_handler: None,
			notification_agent: NotificationAgent::dispatcher(),
		}
	}

	fn update(&mut self, msg: Self::Message) {
		match msg {
			Msg::EndpointFetchResponse(refresh_time, id, r) => {
				let r = match r {
					Ok(articles) => {
						let mut updated_articles = Vec::new();
						for article in articles {
							let article = self.insert_or_update(article);

							updated_articles.push(article as Rc<RefCell<dyn ArticleData>>);
						}

						Ok((updated_articles, None))
					},
					Err(err) => {
						match err {
							Error::UnauthorizedFetch { .. } => {
								self.auth_state = AuthState::NotLoggedIn;
								self.notification_agent.send(NotificationRequest::Notify(
									Some("YouTubeLogin".to_owned()),
									Notification::Login("YouTube".to_owned(), "/proxy/youtube/login".to_owned())
								));

								Ok((Vec::new(), None))
							}
							_ => Err(err),
						}
					}
				};

				self.endpoint_agent.send(EndpointRequest::EndpointFetchResponse(refresh_time, id, r));
			}
		}
	}

	fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
		match msg {
			Request::Auth(auth) => {
				self.auth_state = match auth {
					true => AuthState::LoggedIn,
					false => AuthState::NotLoggedIn,
				};

				if let Some(sidebar_handler) = self.sidebar_handler {
					self.link.respond(sidebar_handler, Response::Sidebar(self.sidebar()));
				}
			},
			Request::AddArticles(refresh_time, endpoint_id, articles) => {
				let mut updated_articles = Vec::new();
				for article in articles.into_iter() {
					let article = self.insert_or_update(article);

					updated_articles.push(article as Rc<RefCell<dyn ArticleData>>);
				}
				self.endpoint_agent.send(EndpointRequest::AddArticles(
					refresh_time,
					endpoint_id,
					updated_articles,
				));

				//self.check_unfetched_articles();
			}
			Request::FetchArticles(refresh_time, id, url) =>
				self.link.send_future(async move {
					Msg::EndpointFetchResponse(refresh_time, id, fetch_videos(url, &get_service_storage("YouTube")).await)
				}),
			Request::Sidebar => {
				self.sidebar_handler = Some(id);
				self.link.respond(id, Response::Sidebar(self.sidebar()));
			},
		}
	}

	fn disconnected(&mut self, id: HandlerId) {
		if Some(id) == self.sidebar_handler {
			self.sidebar_handler = None;
		}
	}
}

impl YouTubeAgent {
	//TODO pub type ArticleRc<A = dyn ArticleData> = Rc<RefCell<A>>
	fn insert_or_update(&mut self, article: Rc<RefCell<YouTubeArticleData>>) -> Rc<RefCell<YouTubeArticleData>> {
		let borrow = article.borrow();
		self.articles.entry(borrow.id.clone())
			.and_modify(|a| a.borrow_mut().update(&borrow))
			.or_insert_with(|| article.clone()).clone()
	}

	fn sidebar(&self) -> Html {
		html! {
			<div class="box">
				<div class="block">
					{"YouTube"}
				</div>
				{ match self.auth_state {
					AuthState::NotLoggedIn => html! {
						<div class="block">
							<a class="button" href="/proxy/youtube/login">{"Login"}</a>
						</div>
					},
					AuthState::LoggedIn => html! {
						{ "Logged in" }
					},
				} }
			</div>
		}
	}
}