use std::{rc::Rc, collections::HashSet};
use yew_agent::{Agent, AgentLink, Context, HandlerId, Bridge};
use yew_agent::utils::store::{StoreWrapper, ReadOnly, Bridgeable};
use js_sys::Date;
use wasm_bindgen::JsValue;

pub mod endpoints;

use crate::articles::{ArticleData};
use crate::services::endpoints::{EndpointStore, StoreRequest as EndpointRequest, EndpointId, EndpointConstructor};
use crate::error::{Result, Error};
use crate::services::twitter::endpoints::{UserTimelineEndpoint, HomeTimelineEndpoint, ListEndpoint, SingleTweetEndpoint};

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
}

impl ArticleData for TweetArticleData {
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
}

pub async fn fetch_tweets(url: &str) -> Result<Vec<Rc<dyn ArticleData>>> {
	let json_str = reqwest::Client::builder().build()?
		.get(format!("http://localhost:8080{}", url))
		//.query(&[("rts", false), ("replies", false)])
		.send().await?
		.text().await?
		.to_string();

	serde_json::from_str(&json_str)
		.map(|value: serde_json::Value|
			value
			.as_array().unwrap()
			.iter()
			.map(|json| Rc::new(TweetArticleData::from(json)) as Rc<dyn ArticleData>)
			.collect()
		)
		.map_err(|err| Error::from(err))
}

pub async fn fetch_tweet(url: &str) -> Result<Rc<dyn ArticleData>> {
	let json_str = reqwest::Client::builder().build()?
		.get(format!("http://localhost:8080{}", url))
		.send().await?
		.text().await?
		.to_string();

	let value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
	Ok(Rc::new(TweetArticleData::from(value)))
}

impl From<&serde_json::Value> for TweetArticleData {
	fn from(json: &serde_json::Value) -> Self {
		let medias_opt = json["extended_entities"]
		.get("media")
		.and_then(|media| media.as_array());
		TweetArticleData {
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
			}
		}
	}
}

impl From<serde_json::Value> for TweetArticleData {
	fn from(json: serde_json::Value) -> Self {
		TweetArticleData::from(&json)
	}
}

pub struct RateLimit {
	pub limit: i32,
	pub remaining: i32,
	pub reset: i32,
}

pub struct TwitterAgent {
	link: AgentLink<Self>,
	endpoint_store: Box<dyn Bridge<StoreWrapper<EndpointStore>>>,
	subscribers: HashSet<HandlerId>,
}

pub enum Request {
	FetchTweets(EndpointId, String),
	FetchTweet(EndpointId, String),
}

pub enum Msg {
	DefaultEndpoint(EndpointId),
	FetchResponse(EndpointId, Result<Vec<Rc<dyn ArticleData>>>),
	EndpointStoreResponse(ReadOnly<EndpointStore>),
}

pub enum Response {
	DefaultEndpoint(EndpointId),
}

impl Agent for TwitterAgent {
	type Reach = Context<Self>;
	type Message = Msg;
	type Input = Request;
	type Output = Response;

	fn create(link: AgentLink<Self>) -> Self {
		let mut endpoint_store = EndpointStore::bridge(link.callback(Msg::EndpointStoreResponse));
		endpoint_store.send(EndpointRequest::InitService("Twitter".to_owned(), vec![
			EndpointConstructor {
				name: "User Timeline",
				param_template: vec!["username"],
				callback: Rc::new(|id, _params| Box::new(UserTimelineEndpoint::new(id, "misabiko_".to_owned()))),
			},
			EndpointConstructor {
				name: "Home Timeline",
				param_template: vec![],
				callback: Rc::new(|id, _params| Box::new(HomeTimelineEndpoint::new(id))),
			},
			EndpointConstructor {
				name: "List",
				param_template: vec!["username", "slug"],
				callback: Rc::new(|id, _params| Box::new(ListEndpoint::new(id, "misabiko".to_owned(), "art".to_owned()))),
			},
			EndpointConstructor {
				name: "Single Tweet",
				param_template: vec!["id"],
				callback: Rc::new(|id, _params| Box::new(SingleTweetEndpoint::new(id, 1471333597189931014))),
			},
		]));

		Self {
			endpoint_store,
			link,
			subscribers: HashSet::new(),
		}
	}

	fn connected(&mut self, id: HandlerId) {
		self.subscribers.insert(id);
	}

	fn disconnected(&mut self, id: HandlerId) {
		self.subscribers.remove(&id);
	}

	fn update(&mut self, msg: Self::Message) {
		match msg {
			Msg::DefaultEndpoint(e) => {
				for sub in self.subscribers.iter() {
					self.link.respond(*sub, Response::DefaultEndpoint(e));
				}
			}
			Msg::FetchResponse(id, r) =>
				self.endpoint_store.send(EndpointRequest::FetchResponse(id, r)),
			Msg::EndpointStoreResponse(_) => {}
		};
	}

	fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
		match msg {
			Request::FetchTweets(id, path) =>
				self.link.send_future(async move {
					Msg::FetchResponse(id, fetch_tweets(&path).await)
				}),
			Request::FetchTweet(id, path) =>
				self.link.send_future(async move {
					Msg::FetchResponse(id, fetch_tweet(&path).await.map(|a| vec![a]))
				})
		}
	}
}