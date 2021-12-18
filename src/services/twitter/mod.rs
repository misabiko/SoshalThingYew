use std::rc::{Rc, Weak};
use std::cell::{RefCell, Ref};
use yew_agent::{Agent, AgentLink, Context, HandlerId, Bridge, Dispatched, Dispatcher};
use yew_agent::utils::store::{StoreWrapper, ReadOnly, Bridgeable};
use js_sys::Date;
use wasm_bindgen::JsValue;
use std::collections::HashMap;

pub mod endpoints;

use crate::articles::{ArticleData, ArticleRefType};
use crate::services::endpoints::{EndpointStore, Request as EndpointRequest, EndpointId, EndpointConstructor, RefreshTime, RateLimit};
use crate::error::{Error, FetchResult};
use crate::services::twitter::endpoints::{UserTimelineEndpoint, HomeTimelineEndpoint, ListEndpoint, SingleTweetEndpoint};
use crate::services::article_actions::{ArticleActionsAgent, ServiceActions, Request as ArticleActionsRequest};

#[derive(Clone, PartialEq)]
pub struct TwitterUser {
	pub username: String,
	pub name: String,
	pub avatar_url: String,
}

pub struct TweetArticleData {
	id: u64,
	text: Option<String>,
	author: TwitterUser,
	creation_time: Date,
	liked: bool,
	retweeted: bool,
	like_count: i64,	//TODO Try casting i64 to i32
	retweet_count: i64,
	media: Vec<String>,
	raw_json: serde_json::Value,
	referenced_article: ArticleRefType,
	marked_as_read: bool,
	hidden: bool,
}

impl ArticleData for TweetArticleData {
	fn service(&self) -> &'static str {
		"Twitter"
	}
	fn id(&self) -> String {
		self.id.clone().to_string()
	}
	fn creation_time(&self) -> Date {
		self.creation_time.clone()
	}
	fn text(&self) -> String {
		self.text.clone().unwrap_or("".to_owned())
	}
	fn author_username(&self) -> String {
		self.author.username.clone()
	}
	fn author_name(&self) -> String {
		self.author.name.clone()
	}
	fn author_avatar_url(&self) -> String {
		self.author.avatar_url.clone()
	}
	fn author_url(&self) -> String {
		format!("https://twitter.com/{}", &self.author.username)
	}
	fn like_count(&self) -> i64 {
		self.like_count.clone()
	}
	fn repost_count(&self) -> i64 {
		self.retweet_count.clone()
	}
	fn liked(&self) -> bool {
		self.liked.clone()
	}
	fn reposted(&self) -> bool {
		self.retweeted.clone()
	}

	fn media(&self) -> Vec<String> {
		self.media.clone()
	}
	fn json(&self) -> serde_json::Value { self.raw_json.clone() }
	fn referenced_article(&self) -> ArticleRefType {
		self.referenced_article.clone()
	}
	fn url(&self) -> String {
		format!("https://twitter.com/{}/status/{}", &self.author_username(), &self.id())
	}

	fn update(&mut self, new: &Ref<dyn ArticleData>) {
		self.liked = new.liked();
		self.retweeted = new.reposted();
		self.like_count = new.like_count();
		self.retweet_count = new.repost_count();
		self.raw_json = new.json();
	}
	fn marked_as_read(&self) -> bool {
		self.marked_as_read.clone()
	}
	fn set_marked_as_read(&mut self, value: bool) {
		self.marked_as_read = value;
	}
	fn hidden(&self) -> bool {
		self.hidden.clone()
	}
	fn set_hidden(&mut self, value: bool) {
		self.hidden = value;
	}
}

impl TweetArticleData {
	fn from(json: &serde_json::Value) -> (Rc<RefCell<Self>>, Option<Rc<RefCell<Self>>>) {
		let referenced_article: Option<(Rc<RefCell<Self>>, bool)> = {
			let referenced = &json["retweeted_status"];
			let quoted = &json["quoted_status"];

			if !referenced.is_null() {
				let parsed = TweetArticleData::from(&referenced.clone());
				if parsed.1.is_some() {
					log::error!("Retweet of a quote/retweet on {:?}??", json["id"]);
				}
				Some((parsed.0, false))
			}else if !quoted.is_null() {
				let parsed = TweetArticleData::from(&quoted.clone());
				if parsed.1.is_some() {
					log::error!("Quote of a quote/retweet on {:?}??\n(actually quite possible but I can't be bothered code it yet)", json["id"]);
				}
				Some((parsed.0, true))
			}else {
				None
			}
		};

		let medias_opt = json["extended_entities"]
			.get("media")
			.and_then(|media| media.as_array());
		let data = Rc::new(RefCell::new(TweetArticleData {
			id: json["id"].as_u64().unwrap(),
			creation_time: json["created_at"].as_str().map(|datetime_str|Date::new(&JsValue::from_str(datetime_str))).unwrap(),
			text: match json["full_text"].as_str() {
				Some(text) => Some(text),
				None => json["text"].as_str()
			}.map(String::from),
			author: TwitterUser {
				username: json["user"]["screen_name"].as_str().unwrap().to_owned(),
				name: json["user"]["name"].as_str().unwrap().to_owned(),
				avatar_url: json["user"]["profile_image_url_https"].as_str().unwrap().to_owned(),
			},
			liked: json["favorited"].as_bool().unwrap_or_default(),
			retweeted: json["retweeted"].as_bool().unwrap_or_default(),
			like_count: json["favorite_count"].as_i64().unwrap(),
			retweet_count: json["retweet_count"].as_i64().unwrap(),
			media: match medias_opt {
				Some(medias) => medias.iter()
					.map(|m|
						m.get("media_url_https")
							.and_then(|url| url.as_str())
							.map(|url| url.to_owned())
					)
					.filter_map(std::convert::identity)
					.collect(),
				None => Vec::new()
			},
			raw_json: json.clone(),
			referenced_article: match &referenced_article {
				None => ArticleRefType::NoRef,
				Some((a, false)) => ArticleRefType::Repost(Rc::downgrade(a) as Weak<RefCell<dyn ArticleData>>),
				Some((a, true)) => ArticleRefType::Quote(Rc::downgrade(a) as Weak<RefCell<dyn ArticleData>>),
			},
			marked_as_read: false,
			hidden: false,
		}));
		(data, referenced_article.map(|(a, _)| a))
	}
}

pub async fn fetch_tweets(url: &str) -> FetchResult<Vec<(Rc<RefCell<TweetArticleData>>, Option<Rc<RefCell<TweetArticleData>>>)>> {
	let response = reqwest::Client::builder().build()?
		.get(format!("http://localhost:8080{}", url))
		.send().await?;

	let headers = response.headers();
	let ratelimit = RateLimit::try_from(headers)?;

	let json_str = response.text().await?.to_string();


	serde_json::from_str(&json_str)
		.map(|value: serde_json::Value|
			(value
			.as_array().unwrap()
			.iter()
			.map(|json| TweetArticleData::from(json))
			.collect(),
			 Some(ratelimit))
		)
		.map_err(|err| Error::from(err))
}

pub async fn fetch_tweet(url: &str) -> FetchResult<(Rc<RefCell<TweetArticleData>>, Option<Rc<RefCell<TweetArticleData>>>)> {
	let response = reqwest::Client::builder().build()?
		.get(format!("http://localhost:8080{}", url))
		.send().await?;

	let headers = response.headers();
	let ratelimit = RateLimit::try_from(headers)?;

	let json_str = response.text().await?.to_string();

	let value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
	Ok((TweetArticleData::from(&value), Some(ratelimit)))
}

//TODO Receive TwitterUser
pub enum AuthMode {
	NotLoggedIn,
	LoggedIn(TwitterUser)
}

pub struct TwitterAgent {
	link: AgentLink<Self>,
	endpoint_store: Box<dyn Bridge<StoreWrapper<EndpointStore>>>,
	#[allow(dead_code)]
	actions_agent: Dispatcher<ArticleActionsAgent>,
	articles: HashMap<u64, Rc<RefCell<TweetArticleData>>>
	//auth_mode: AuthMode,
}

pub enum Request {
	FetchTweets(RefreshTime, EndpointId, String),
	FetchTweet(RefreshTime, EndpointId, String),
}

pub enum Msg {
	FetchResponse(HandlerId, FetchResult<Vec<Rc<RefCell<TweetArticleData>>>>),
	EndpointFetchResponse(RefreshTime, EndpointId, FetchResult<Vec<(Rc<RefCell<TweetArticleData>>, Option<Rc<RefCell<TweetArticleData>>>)>>),
	EndpointStoreResponse(ReadOnly<EndpointStore>),
	Like(HandlerId, Weak<RefCell<dyn ArticleData>>),
	Retweet(HandlerId, Weak<RefCell<dyn ArticleData>>),
}

impl Agent for TwitterAgent {
	type Reach = Context<Self>;
	type Message = Msg;
	type Input = Request;
	type Output = ();

	fn create(link: AgentLink<Self>) -> Self {
		let mut endpoint_store = EndpointStore::bridge(link.callback(Msg::EndpointStoreResponse));
		endpoint_store.send(EndpointRequest::InitService("Twitter".to_owned(), vec![
			EndpointConstructor {
				name: "User Timeline",
				param_template: vec!["username"],
				callback: Rc::new(|id, params| Box::new(UserTimelineEndpoint::from_json(id, params))),
			},
			EndpointConstructor {
				name: "Home Timeline",
				param_template: vec![],
				callback: Rc::new(|id, _params| Box::new(HomeTimelineEndpoint::new(id))),
			},
			EndpointConstructor {
				name: "List",
				param_template: vec!["username", "slug"],
				callback: Rc::new(|id, params| Box::new(ListEndpoint::from_json(id, params))),
			},
			EndpointConstructor {
				name: "Single Tweet",
				param_template: vec!["id"],
				callback: Rc::new(|id, params| Box::new(SingleTweetEndpoint::from_json(id, params))),
			},
		]));

		let mut actions_agent = ArticleActionsAgent::dispatcher();
		actions_agent.send(ArticleActionsRequest::Init("Twitter", ServiceActions {
			like: link.callback(|(id, article)| Msg::Like(id, article)),
			repost: link.callback(|(id, article)| Msg::Retweet(id, article)),
		}));

		Self {
			endpoint_store,
			link,
			actions_agent,
			articles: HashMap::new(),
			//auth_mode: AuthMode::NotLoggedIn,
		}
	}

	fn update(&mut self, msg: Self::Message) {
		match msg {
			Msg::EndpointFetchResponse(refresh_time, id, r) => {
				let mut valid_rc = Vec::new();
				if let Ok((articles, _)) = &r {
					for (article, ref_article_opt) in articles {
						let borrow = article.borrow();
						let valid_a_rc = self.articles.entry(borrow.id)
							.and_modify(|a| a.borrow_mut().update(&(borrow as Ref<dyn ArticleData>)))
							.or_insert_with(|| article.clone()).clone();

						if let Some(ref_article) = ref_article_opt {
							let ref_borrow = ref_article.borrow();
							let valid_a_ref_rc = self.articles.entry(ref_borrow.id)
								.and_modify(|a| a.borrow_mut().update(&(ref_borrow as Ref<dyn ArticleData>)))
								.or_insert_with(|| ref_article.clone()).clone();

							valid_rc.push((valid_a_rc, Some(valid_a_ref_rc)));
						}else {
							valid_rc.push((valid_a_rc, None));
						}
					}
				}
				self.endpoint_store.send(EndpointRequest::EndpointFetchResponse(
					refresh_time,
					id,
					r.map(move |(_, ratelimit)|
						(
							valid_rc.into_iter()
								.map(|(article, ref_article_opt)|
									(
										article as Rc<RefCell<dyn ArticleData>>,
									 	ref_article_opt.map(|a| a as Rc<RefCell<dyn ArticleData>>),
									)
								).collect(),
						 	ratelimit
						))
				));
			}
			Msg::FetchResponse(id, r) => {
				if let Ok((articles, _)) = &r {
					let mut valid_rc = Vec::new();
					for article in articles {
						let borrow = article.borrow();
						let updated = self.articles.entry(borrow.id)
							.and_modify(|a| a.borrow_mut().update(&(borrow as Ref<dyn ArticleData>)))
							.or_insert_with(|| article.clone());

						valid_rc.push(Rc::downgrade(updated) as Weak<RefCell<dyn ArticleData>>);
					}

					self.actions_agent.send(ArticleActionsRequest::Callback(valid_rc));
				}
			}
			Msg::EndpointStoreResponse(_) => {}
			Msg::Like(id, article) => {
				let strong = article.upgrade().unwrap();
				let borrow = strong.borrow();

				//TODO Support liking quotes
				if let ArticleRefType::NoRef = borrow.referenced_article() {
					let path = format!("/proxy/twitter/{}/{}", if borrow.liked() { "unlike" } else { "like" }, borrow.id());
					self.link.send_future(async move {
						Msg::FetchResponse(id, fetch_tweet(&path).await.map(|a| (match a.0 {
							(article, Some(ref_article)) => vec![article, ref_article],
							(article, None) => vec![article],
						}, a.1)))
					})
				}
			}
			Msg::Retweet(id, article) => {
				let strong = article.upgrade().unwrap();
				let borrow = strong.borrow();

				//TODO Support retweeting quotes
				if let ArticleRefType::NoRef = borrow.referenced_article() {
					let path = format!("/proxy/twitter/{}/{}", if borrow.reposted() { "unretweet" } else { "retweet" }, borrow.id());
					self.link.send_future(async move {
						Msg::FetchResponse(id, fetch_tweet(&path).await.map(|a| (match a.0 {
							(article, Some(ref_article)) => vec![article, ref_article],
							(article, None) => vec![article],
						}, a.1)))
					})
				}
			}
		};
	}

	fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
		match msg {
			Request::FetchTweets(refresh_time, id, path) =>
				self.link.send_future(async move {
					Msg::EndpointFetchResponse(refresh_time, id, fetch_tweets(&path).await)
				}),
			Request::FetchTweet(refresh_time, id, path) =>
				self.link.send_future(async move {
					Msg::EndpointFetchResponse(refresh_time, id, fetch_tweet(&path).await.map(|a| (vec![a.0], a.1)))
				})
		}
	}
}