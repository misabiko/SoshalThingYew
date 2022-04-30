use std::collections::HashMap;
use std::rc::Rc;
use reqwest::Url;
use yew_agent::{Agent, AgentLink, Context, Dispatcher, Dispatched, HandlerId};
use yew::prelude::*;

mod article;
mod endpoints;

use article::YouTubeArticleData;
use crate::articles::ArticleRc;
use crate::error::{Result, Error};
use crate::notifications::{Notification, NotificationAgent, NotificationRequest};
use crate::services::{
	service,
	RefreshTime,
	endpoint_agent::{EndpointAgent, EndpointConstructor, EndpointId, EndpointRequest},
	article_actions::{ArticleActionsAgent, ServiceActions, ArticleActionsRequest},
	endpoint_agent::EndpointConstructorCollection,
	storages::get_service_storage,
	youtube::endpoints::{fetch_videos, PlaylistEndpoint},
};

#[derive(Debug)]
enum AuthState {
	NotLoggedIn,
	LoggedIn,
}

//TODO type<A> ServiceArticles = HashMap<A::Id, Rc<RefCell<A>>>
#[service("YouTube", YouTubeArticleData, String)]
pub struct YouTubeAgent {
	link: AgentLink<Self>,
	endpoint_agent: Dispatcher<EndpointAgent>,
	_actions_agent: Dispatcher<ArticleActionsAgent>,
	auth_state: AuthState,
	sidebar_handler: Option<HandlerId>,
	notification_agent: Dispatcher<NotificationAgent>,
}

pub enum YouTubeMsg {
	EndpointFetchResponse(RefreshTime, EndpointId, Result<Vec<ArticleRc<YouTubeArticleData>>>),
}

pub enum YouTubeRequest {
	Auth(bool),
	AddArticles(RefreshTime, EndpointId, Vec<ArticleRc<YouTubeArticleData>>),
	FetchArticles(RefreshTime, EndpointId, Url),
	Sidebar,
}

pub enum YouTubeResponse {
	Sidebar(Html),
}

type Msg = YouTubeMsg;
type Request = YouTubeRequest;
type Response = YouTubeResponse;

impl Agent for YouTubeAgent {
	type Reach = Context<Self>;
	type Message = Msg;
	type Input = Request;
	type Output = Response;

	fn create(link: AgentLink<Self>) -> Self {
		let mut endpoint_agent = EndpointAgent::dispatcher();
		endpoint_agent.send(EndpointRequest::InitService(
			SERVICE_INFO.name,
			EndpointConstructorCollection {
				constructors: vec![
					EndpointConstructor {
						name: "Playlist",
						param_template: vec![
							("id", serde_json::Value::String("".to_owned()))
						],
						callback: Rc::new(|id, params| Box::new(PlaylistEndpoint::from_json(id, params))),
					},
				],
				user_endpoint_index: None,
			}));

		let mut _actions_agent = ArticleActionsAgent::dispatcher();
		_actions_agent.send(ArticleActionsRequest::Init(SERVICE_INFO.name, ServiceActions {
			like: None,
			repost: None,
			fetch_data: None,
		}));

		Self {
			endpoint_agent,
			link,
			_actions_agent,
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

							updated_articles.push(article as ArticleRc);
						}

						Ok((updated_articles, None))
					},
					Err(err) => {
						match err {
							Error::UnauthorizedFetch { .. } => {
								self.auth_state = AuthState::NotLoggedIn;
								self.notification_agent.send(NotificationRequest::Notify(
									Some("YouTubeLogin".to_owned()),
									Notification::Login(SERVICE_INFO.name.to_owned(), "/proxy/youtube/login".to_owned())
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

					updated_articles.push(article as ArticleRc);
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
					Msg::EndpointFetchResponse(refresh_time, id, fetch_videos(url, &get_service_storage(SERVICE_INFO.name)).await)
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
	fn insert_or_update(&mut self, article: ArticleRc<YouTubeArticleData>) -> ArticleRc<YouTubeArticleData> {
		let borrow = article.borrow();
		self.articles.entry(borrow.id.clone())
			.and_modify(|a| a.borrow_mut().update(&borrow))
			.or_insert_with(|| article.clone()).clone()
	}

	fn sidebar(&self) -> Html {
		html! {
			<div class="box">
				<div class="block">
					{SERVICE_INFO.name}
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