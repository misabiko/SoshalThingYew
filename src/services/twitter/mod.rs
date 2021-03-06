use yew::prelude::*;
use std::rc::Rc;
use yew_agent::{Agent, AgentLink, Context, HandlerId, Dispatched, Dispatcher};
use std::collections::HashMap;
use reqwest::{StatusCode, Url};

pub mod endpoints;
pub mod article;

pub use article::TweetArticleData;
use article::StrongArticleRefType;
use crate::articles::{ArticleRc, ArticleRefType, ArticleWeak};
use crate::{base_url, SearchEndpoint};
use crate::notifications::{Notification, NotificationAgent, NotificationRequest};
use crate::services::{
	service,
	RateLimit,
	endpoint_agent::{EndpointAgent, EndpointRequest, EndpointId, EndpointConstructor, EndpointConstructorCollection, RefreshTime},
	article_actions::{ArticleActionsAgent, ServiceActions, ArticleActionsRequest},
	twitter::endpoints::*,
};
use crate::error::{Error, RatelimitedResult};
use crate::services::storages::{get_service_storage, ServiceStorage};

pub async fn fetch_tweets(url: Url, storage: &ServiceStorage) -> RatelimitedResult<Vec<(ArticleRc<TweetArticleData>, Vec<StrongArticleRefType>)>> {
	let response = reqwest::Client::builder()
		//.timeout(Duration::from_secs(10))
		.build()?
		.get(url)
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
			(match value.as_array() {
				Some(array) => array.iter().map(|json| TweetArticleData::from(json, storage)).map(|(rc, refs, _)| (rc, refs)).collect(),
				None => {
					let (rc, refs, _) = TweetArticleData::from(&value, storage);
					vec![(rc, refs)]
				},
			},
			 Some(ratelimit))
		)
		.map_err(|err| Error::from(err))
}

#[derive(Debug)]
enum AuthState {
	NotLoggedIn,
	LoggedIn(u64)
}

#[service("Twitter", TweetArticleData, u64)]
pub struct TwitterAgent {
	link: AgentLink<Self>,
	endpoint_agent: Dispatcher<EndpointAgent>,
	actions_agent: Dispatcher<ArticleActionsAgent>,
	auth_state: AuthState,
	sidebar_handler: Option<HandlerId>,
	notification_agent: Dispatcher<NotificationAgent>,
}

pub enum TwitterMsg {
	FetchResponse(HandlerId, RatelimitedResult<Vec<(ArticleRc<TweetArticleData>, Vec<StrongArticleRefType>)>>),
	EndpointFetchResponse(RefreshTime, EndpointId, RatelimitedResult<Vec<(ArticleRc<TweetArticleData>, Vec<StrongArticleRefType>)>>),
	Like(HandlerId, ArticleWeak),
	Retweet(HandlerId, ArticleWeak),
}

pub enum TwitterRequest {
	Auth(Option<String>),
	Sidebar,
	FetchTweets(RefreshTime, EndpointId, Url),
	FetchTweet(RefreshTime, EndpointId, Url),
}

pub enum TwitterResponse {
	Sidebar(Html),
}

type Msg = TwitterMsg;
type Request = TwitterRequest;
type Response = TwitterResponse;

impl Agent for TwitterAgent {
	type Reach = Context<Self>;
	type Message = Msg;
	type Input = Request;
	type Output = Response;

	fn create(link: AgentLink<Self>) -> Self {
		let mut endpoint_agent = EndpointAgent::dispatcher();
		endpoint_agent.send(EndpointRequest::InitService(
			SERVICE_INFO.name,
			EndpointConstructorCollection {
				 //TODO Needs to sync other eq_storage when modifying this
				constructors: vec![
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
						name: "Likes",
						param_template: vec![
							("username", serde_json::Value::String("".to_owned())),
						],
						callback: Rc::new(|id, params| Box::new(LikesEndpoint::from_json(id, params))),
					},
					EndpointConstructor {
						name: "Single Tweet",
						param_template: vec![
							("id", serde_json::Value::String("".to_owned())),
						],
						callback: Rc::new(|id, params| Box::new(SingleTweetEndpoint::from_json(id, params))),
					},
					EndpointConstructor {
						name: "Search",
						param_template: vec![
							("query", serde_json::Value::String("".to_owned())),
						],
						callback: Rc::new(|id, params| Box::new(SearchEndpoint::from_json(id, params))),
					},
				],
				user_endpoint_index: Some(1)
			}
		));

		let mut actions_agent = ArticleActionsAgent::dispatcher();
		actions_agent.send(ArticleActionsRequest::Init(SERVICE_INFO.name, ServiceActions {
			like: Some(link.callback(|(id, article)| Msg::Like(id, article))),
			repost: Some(link.callback(|(id, article)| Msg::Retweet(id, article))),
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
					Ok((articles, ratelimit)) => {
						let mut updated_articles = Vec::new();
						for (article, ref_articles) in articles {
							let article = self.insert_or_update(article, ref_articles);
							updated_articles.push(article as ArticleRc);
						}

						Ok((updated_articles, ratelimit))
					},
					Err(err) => {
						match err {
							Error::UnauthorizedFetch { .. } => {
								self.auth_state = AuthState::NotLoggedIn;
								self.notification_agent.send(NotificationRequest::Notify(
									Some("TwitterLogin".to_owned()),
									Notification::Login(SERVICE_INFO.name.to_owned(), "/proxy/twitter/login".to_owned())
								));

								Ok((Vec::new(), None))
							}
							_ => Err(err),
						}
					}
				};

				self.endpoint_agent.send(EndpointRequest::EndpointFetchResponse(refresh_time, id, r));
			}
			Msg::FetchResponse(_id, r) => {
				if let Ok((articles, _)) = r {
					let articles = articles.into_iter()
						.map(|(article, ref_articles)| {
							let article = self.insert_or_update(article, ref_articles);
							Rc::downgrade(&article) as ArticleWeak
						})
						.collect();

					self.actions_agent.send(ArticleActionsRequest::RedrawTimelines(articles));
				}
			}
			Msg::Like(id, article) => {
				let strong = article.upgrade().unwrap();
				let borrow = strong.borrow();

				let url = Url::parse(&format!("{}/proxy/twitter/{}/{}", base_url(), if borrow.liked() { "unlike" } else { "like" }, borrow.id())).unwrap();

				self.link.send_future(async move {
					Msg::FetchResponse(id, fetch_tweets(url, &get_service_storage(SERVICE_INFO.name)).await)
				})
			}
			Msg::Retweet(id, article) => {
				let strong = article.upgrade().unwrap();
				let borrow = strong.borrow();

				let url = Url::parse(&format!("{}/proxy/twitter/{}/{}", base_url(), if borrow.reposted() { "unretweet" } else { "retweet" }, borrow.id())).unwrap();

				self.link.send_future(async move {
					Msg::FetchResponse(id, fetch_tweets(url, &get_service_storage(SERVICE_INFO.name)).await)
				})
			}
		};
	}

	fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
		match msg {
			Request::Auth(auth) => {
				self.auth_state = match auth {
					Some(auth) => AuthState::LoggedIn(auth.parse().expect("parsing twitter user id")),
					None => AuthState::NotLoggedIn,
				};

				if let Some(sidebar_handler) = self.sidebar_handler {
					self.link.respond(sidebar_handler, Response::Sidebar(self.sidebar()));
				}
			},
			Request::Sidebar => {
				self.sidebar_handler = Some(id);
				self.link.respond(id, Response::Sidebar(self.sidebar()));
			},
			Request::FetchTweets(refresh_time, id, url) =>
				self.link.send_future(async move {
					Msg::EndpointFetchResponse(refresh_time, id, fetch_tweets(url, &get_service_storage(SERVICE_INFO.name)).await)
				}),
			Request::FetchTweet(refresh_time, id, url) =>
				self.link.send_future(async move {
					Msg::EndpointFetchResponse(refresh_time, id, fetch_tweets(url, &get_service_storage(SERVICE_INFO.name)).await)
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
					{SERVICE_INFO.name}
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

	fn insert_or_update(&mut self, article: ArticleRc<TweetArticleData>, ref_articles: Vec<StrongArticleRefType>) -> ArticleRc<TweetArticleData> {
		let borrow = article.borrow();
		let article = self.articles.entry(borrow.id)
			.and_modify(|a| a.borrow_mut().update(&borrow))
			.or_insert_with(|| article.clone()).clone();
		drop(borrow);

		for (i, ref_article) in ref_articles.into_iter().enumerate() {
			match ref_article {
				StrongArticleRefType::Reposted(a) => {
					let ref_article = self.insert_or_update(a, vec![]);
					let mut borrow_mut = article.borrow_mut();
					borrow_mut.referenced_articles[i] = ArticleRefType::Reposted(Rc::downgrade(&ref_article));
				}
				StrongArticleRefType::Quote(a) => {
					let ref_article = self.insert_or_update(a, vec![]);
					let mut borrow_mut = article.borrow_mut();
					borrow_mut.referenced_articles[i] = ArticleRefType::Quote(Rc::downgrade(&ref_article));
				}
				StrongArticleRefType::RepostedQuote(a, q) => {
					let ref_quote = self.insert_or_update(a, vec![]);
					let mut borrow_mut = article.borrow_mut();
					borrow_mut.referenced_articles[i] = ArticleRefType::Quote(Rc::downgrade(&ref_quote));

					let ref_article = self.insert_or_update(q, vec![]);
					let mut quote_borrow_mut = ref_quote.borrow_mut();
					quote_borrow_mut.referenced_articles[i] = ArticleRefType::Quote(Rc::downgrade(&ref_article));
				}
				StrongArticleRefType::Reply(a) => {
					let ref_article = self.insert_or_update(a, vec![]);
					let mut borrow_mut = article.borrow_mut();
					borrow_mut.referenced_articles[i] = ArticleRefType::Reply(Rc::downgrade(&ref_article));
				}
			}
		}

		article
	}
}