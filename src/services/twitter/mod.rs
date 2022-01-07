use yew::prelude::*;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use yew_agent::{Agent, AgentLink, Context, HandlerId, Dispatched, Dispatcher};
use std::collections::{HashMap, HashSet};
use gloo_storage::Storage;
use reqwest::StatusCode;

pub mod endpoints;
mod article;

pub use article::TweetArticleData;
use article::StrongArticleRefType;
use crate::articles::{ArticleData, ArticleRefType};
use crate::base_url;
use crate::services::{
	RateLimit,
	endpoint_agent::{EndpointAgent, Request as EndpointRequest, EndpointId, EndpointConstructor, EndpointConstructors, RefreshTime},
	article_actions::{ArticleActionsAgent, ServiceActions, Request as ArticleActionsRequest},
	twitter::endpoints::{UserTimelineEndpoint, HomeTimelineEndpoint, ListEndpoint, SingleTweetEndpoint},
};
use crate::error::{Error, RatelimitedResult};
use crate::services::storages::SoshalSessionStorage;

pub async fn fetch_tweets(url: &str, marked_as_read: &HashSet<u64>) -> RatelimitedResult<Vec<(Rc<RefCell<TweetArticleData>>, StrongArticleRefType)>> {
	let response = reqwest::Client::builder().build()?
		.get(format!("{}{}", base_url(), url))
		.send().await?
		.error_for_status()
		.map_err(|err| if let Some(StatusCode::UNAUTHORIZED) = err.status() {
			Error::UnauthorizedFetch {
				message: None,
				error: err.into(),
				article_ids: vec![],
			}
		}else {
			err.into()
		})?;

	let headers = response.headers();
	let ratelimit = RateLimit::try_from(headers)?;

	let json_str = response.text().await?.to_string();


	serde_json::from_str(&json_str)
		.map(|value: serde_json::Value|
			(value
			.as_array().unwrap()
			.iter()
			.map(|json| TweetArticleData::from(json, &marked_as_read))
			.collect(),
			 Some(ratelimit))
		)
		.map_err(|err| Error::from(err))
}

pub async fn fetch_tweet(url: &str, marked_as_read: &HashSet<u64>) -> RatelimitedResult<(Rc<RefCell<TweetArticleData>>, StrongArticleRefType)> {
	let response = reqwest::Client::builder().build()?
		.get(format!("{}{}", base_url(), url))
		.send().await?
		.error_for_status()
		.map_err(|err| if let Some(StatusCode::UNAUTHORIZED) = err.status() {
			Error::UnauthorizedFetch {
				message: None,
				error: err.into(),
				article_ids: vec![],
			}
		}else {
			err.into()
		})?;

	let headers = response.headers();
	let ratelimit = RateLimit::try_from(headers)?;

	let json_str = response.text().await?.to_string();

	let value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
	Ok((TweetArticleData::from(&value, &marked_as_read), Some(ratelimit)))
}

#[derive(Debug)]
pub enum AuthState {
	NotLoggedIn,
	LoggedIn(u64)
}

pub struct TwitterAgent {
	link: AgentLink<Self>,
	endpoint_agent: Dispatcher<EndpointAgent>,
	actions_agent: Dispatcher<ArticleActionsAgent>,
	articles: HashMap<u64, Rc<RefCell<TweetArticleData>>>,
	cached_marked_as_read: HashSet<u64>,
	auth_state: AuthState,
	sidebar_handler: Option<HandlerId>,
}

pub enum Msg {
	FetchResponse(HandlerId, RatelimitedResult<Vec<Rc<RefCell<TweetArticleData>>>>),
	EndpointFetchResponse(RefreshTime, EndpointId, RatelimitedResult<Vec<(Rc<RefCell<TweetArticleData>>, StrongArticleRefType)>>),
	Like(HandlerId, Weak<RefCell<dyn ArticleData>>),
	Retweet(HandlerId, Weak<RefCell<dyn ArticleData>>),
}

pub enum Request {
	Auth(Option<String>),
	Sidebar,
	FetchTweets(RefreshTime, EndpointId, String),
	FetchTweet(RefreshTime, EndpointId, String),
}

pub enum Response {
	Sidebar(Html),
}

impl Agent for TwitterAgent {
	type Reach = Context<Self>;
	type Message = Msg;
	type Input = Request;
	type Output = Response;

	fn create(link: AgentLink<Self>) -> Self {
		let mut endpoint_agent = EndpointAgent::dispatcher();
		endpoint_agent.send(EndpointRequest::InitService(
			"Twitter".to_owned(),
			 EndpointConstructors {
				 //TODO Needs to sync other eq_storage when modifying this
				endpoint_types: vec![
					EndpointConstructor {
						name: "Home Timeline",
						param_template: vec![],
						callback: Rc::new(|id, _params| Box::new(HomeTimelineEndpoint::new(id))),
					},
					EndpointConstructor {
						name: "User Timeline",
						param_template: vec![
							("username", serde_json::Value::String("".to_owned())),
							("include_retweets", serde_json::Value::Bool(true)),
							("include_replies", serde_json::Value::Bool(true)),
						],
						callback: Rc::new(|id, params| Box::new(UserTimelineEndpoint::from_json(id, params))),
					},
					EndpointConstructor {
						name: "List",
						param_template: vec![
							("username", serde_json::Value::String("".to_owned())),
							("slug", serde_json::Value::String("".to_owned())),
						],
						callback: Rc::new(|id, params| Box::new(ListEndpoint::from_json(id, params))),
					},
					EndpointConstructor {
						name: "Single Tweet",
						param_template: vec![
							("id", serde_json::Value::String("".to_owned())),
						],
						callback: Rc::new(|id, params| Box::new(SingleTweetEndpoint::from_json(id, params))),
					},
				],
				user_endpoint: Some(1)
			}
		));

		let mut actions_agent = ArticleActionsAgent::dispatcher();
		actions_agent.send(ArticleActionsRequest::Init("Twitter", ServiceActions {
			like: Some(link.callback(|(id, article)| Msg::Like(id, article))),
			repost: Some(link.callback(|(id, article)| Msg::Retweet(id, article))),
			fetch_data: None,
		}));

		let session_storage: Option<SoshalSessionStorage> = gloo_storage::SessionStorage::get("SoshalThingYew").ok();
		let cached_marked_as_read = match &session_storage.as_ref().map(|s| &s.services).and_then(|s| s.get("Twitter")) {
			Some(storage) => storage.articles_marked_as_read.iter().map(|id| id.parse().unwrap()).collect(),
			None => HashSet::new(),
		};

		Self {
			endpoint_agent,
			link,
			actions_agent,
			articles: HashMap::new(),
			cached_marked_as_read,
			auth_state: AuthState::NotLoggedIn,
			sidebar_handler: None,
		}
	}

	fn update(&mut self, msg: Self::Message) {
		match msg {
			Msg::EndpointFetchResponse(refresh_time, id, r) => {
				let mut valid_rc = Vec::new();

				match &r {
					Ok((articles, _)) => {
						for (article, ref_article) in articles {
							let borrow = article.borrow();
							let valid_a_rc = self.articles.entry(borrow.id)
								.and_modify(|a| a.borrow_mut().update(&borrow))
								.or_insert_with(|| article.clone()).clone();

							match ref_article {
								StrongArticleRefType::Repost(a) | StrongArticleRefType::Quote(a) => {
									let ref_borrow = a.borrow();
									self.articles.entry(ref_borrow.id)
										.and_modify(|a| a.borrow_mut().update(&ref_borrow))
										.or_insert_with(|| a.clone());
								}
								StrongArticleRefType::QuoteRepost(a, q) => {
									let a_borrow = a.borrow();
									self.articles.entry(a_borrow.id)
										.and_modify(|a| a.borrow_mut().update(&a_borrow))
										.or_insert_with(|| a.clone());

									let q_borrow = q.borrow();
									self.articles.entry(q_borrow.id)
										.and_modify(|a| a.borrow_mut().update(&q_borrow))
										.or_insert_with(|| q.clone());
								}
								_ => {},
							};

							valid_rc.push(valid_a_rc);
						}
					},
					Err(err) => {
						if let Error::UnauthorizedFetch { .. } = err {
							self.auth_state = AuthState::NotLoggedIn;
						}
					}
				}

				self.endpoint_agent.send(EndpointRequest::EndpointFetchResponse(
					refresh_time,
					id,
					r.map(move |(_, ratelimit)|
						(
							valid_rc.into_iter()
								.map(|article| article as Rc<RefCell<dyn ArticleData>>)
								.collect(),
						 	ratelimit
						))
				));
			}
			Msg::FetchResponse(_id, r) => {
				if let Ok((articles, _)) = &r {
					let mut valid_rc = Vec::new();
					for article in articles {
						let borrow = article.borrow();
						let updated = self.articles.entry(borrow.id)
							.and_modify(|a| a.borrow_mut().update(&borrow))
							.or_insert_with(|| article.clone());

						valid_rc.push(Rc::downgrade(updated) as Weak<RefCell<dyn ArticleData>>);
					}

					self.actions_agent.send(ArticleActionsRequest::RedrawTimelines(valid_rc));
				}
			}
			Msg::Like(id, article) => {
				let strong = article.upgrade().unwrap();
				let borrow = strong.borrow();

				if let ArticleRefType::NoRef | ArticleRefType::Quote(_) = borrow.referenced_article() {
					let path = format!("/proxy/twitter/{}/{}", if borrow.liked() { "unlike" } else { "like" }, borrow.id());
					let marked_as_read = self.cached_marked_as_read.clone();

					self.link.send_future(async move {
						Msg::FetchResponse(id, fetch_tweet(&path, &marked_as_read).await.map(|(articles, ratelimit)| (vec![articles.0], ratelimit)))
					})
				}
			}
			Msg::Retweet(id, article) => {
				let strong = article.upgrade().unwrap();
				let borrow = strong.borrow();

				if let ArticleRefType::NoRef | ArticleRefType::Quote(_) = borrow.referenced_article() {
					let path = format!("/proxy/twitter/{}/{}", if borrow.reposted() { "unretweet" } else { "retweet" }, borrow.id());
					let marked_as_read = self.cached_marked_as_read.clone();

					self.link.send_future(async move {
						Msg::FetchResponse(id, fetch_tweet(&path, &marked_as_read).await.map(|(articles, ratelimit)| (vec![articles.0], ratelimit)))
					})
				}
			}
		};
	}

	fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
		//TODO Use storage
		let marked_as_read = self.cached_marked_as_read.clone();
		match msg {
			Request::Auth(auth) => {
				self.auth_state = match auth {
					Some(auth) => AuthState::LoggedIn(auth.parse().expect("parsing twitter user id")),
					None => AuthState::NotLoggedIn,
				};
				log::debug!("Twitter auth: {:?}", &self.auth_state);
				if let Some(sidebar_handler) = self.sidebar_handler {
					self.link.respond(sidebar_handler, Response::Sidebar(self.sidebar()));
				}
			},
			Request::Sidebar => {
				self.sidebar_handler = Some(id);
				self.link.respond(id, Response::Sidebar(self.sidebar()));
			},
			Request::FetchTweets(refresh_time, id, path) =>
				self.link.send_future(async move {
					Msg::EndpointFetchResponse(refresh_time, id, fetch_tweets(&path, &marked_as_read).await)
				}),
			Request::FetchTweet(refresh_time, id, path) =>
				self.link.send_future(async move {
					Msg::EndpointFetchResponse(refresh_time, id, fetch_tweet(&path, &marked_as_read).await.map(|a| (vec![a.0], a.1)))
				})
		}
	}

	fn disconnected(&mut self, id: HandlerId) {
		if Some(id) == self.sidebar_handler {
			self.sidebar_handler = None;
		}
	}
}

impl TwitterAgent {
	fn sidebar(&self) -> Html {
		html! {
			<div class="box">
				<div class="block">
					{"Twitter"}
				</div>
				{ match self.auth_state {
					AuthState::NotLoggedIn => html! {
						<div class="block">
							<a class="button" href="/proxy/twitter/login">{"Login"}</a>
						</div>
					},
					AuthState::LoggedIn(id) => html! {
						{ format!("Logged with id {}", id) }
					},
				} }
			</div>
		}
	}
}