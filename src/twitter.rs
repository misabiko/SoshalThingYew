use std::{rc::Rc, collections::HashSet};
use yew_agent::{Agent, AgentLink, Context, HandlerId, Dispatched, Dispatcher};
use js_sys::Date;
use wasm_bindgen::JsValue;

use crate::articles::{sort_by_id, SocialArticleData};
use crate::endpoints::{EndpointAgent, Endpoint, Request as EndpointRequest, EndpointId};
use crate::error::{Result, Error};

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

impl SocialArticleData for TweetArticleData {
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

pub async fn fetch_tweets(url: &str) -> Result<Vec<Rc<dyn SocialArticleData>>> {
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
			.map(|json| Rc::new(TweetArticleData::from(json)) as Rc<dyn SocialArticleData>)
			.collect()
		)
		.map_err(|err| Error::from(err))
}

pub async fn fetch_tweet(url: &str) -> Result<Rc<dyn SocialArticleData>> {
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
	//endpoints: Vec<Rc<dyn Endpoint>>,
	endpoint_agent: Dispatcher<EndpointAgent>,
	subscribers: HashSet<HandlerId>,
}

pub enum Request {
	//UpdateRateLimit(RateLimit),
	FetchTweets(EndpointId, String),
	FetchTweet(EndpointId, String),
}

pub enum Msg {
	Init,
	DefaultEndpoint(EndpointId),
	FetchResponse(EndpointId, Result<Vec<Rc<dyn SocialArticleData>>>)
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
		log::debug!("Creating TwitterAgent");
		link.send_message(Msg::Init);

		Self {
			link,
			endpoint_agent: EndpointAgent::dispatcher(),
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
			Msg::Init => {
				/*let callback = self.link.callback(Msg::DefaultEndpoint);
				self.endpoint_agent.send(
					EndpointRequest::AddEndpoint(Box::new(move |id| {
						callback.emit(id);
						Box::new(ArtEndpoint {
							id,
							agent: TwitterAgent::dispatcher(),
						})
					}))
				)*/
			},
			Msg::DefaultEndpoint(e) => {
				for sub in self.subscribers.iter() {
					self.link.respond(*sub, Response::DefaultEndpoint(e));
				}
			}
			Msg::FetchResponse(id, r) =>
				self.endpoint_agent.send(EndpointRequest::FetchResponse(id, r))
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

pub struct UserTimelineEndpoint {
	id: EndpointId,
	username: String,
	articles: Vec<Rc<dyn SocialArticleData>>,
	agent: Dispatcher<TwitterAgent>
}

impl UserTimelineEndpoint {
	pub fn new(id: EndpointId, username: String) -> Self {
		Self {
			id,
			username,
			articles: Vec::new(),
			agent: TwitterAgent::dispatcher(),
		}
	}
}

impl Endpoint for UserTimelineEndpoint {
	fn name(&self) -> String {
		format!("{} User Timeline Endpoint", &self.username).to_owned()
	}

	fn id(&self) -> &EndpointId {
		&self.id
	}

	fn add_articles(&mut self, articles: Vec<Rc<dyn SocialArticleData>>)  {
		for a in articles {
			if !self.articles.iter().any(|existing| existing.id() == a.id()) {
				self.articles.push(a);
			}
		}
		self.articles.sort_by(|a, b| b.id().partial_cmp(&a.id()).unwrap())
	}

	fn refresh(&mut self) {
		let id = self.id().clone();
		self.agent.send(Request::FetchTweets(
			id,
			format!("/proxy/twitter/user/{}", &self.username)
		))
	}

	fn load_bottom(&mut self) {
		match self.articles.last() {
			Some(last_id) => {
				let id = self.id().clone();
				self.agent.send(Request::FetchTweets(
					id,
					format!("/proxy/twitter/user/{}?max_id={}", &self.username, &last_id.id())
				))
			}
			None => self.refresh()
		}
	}
}

pub struct HomeTimelineEndpoint {
	id: EndpointId,
	articles: Vec<Rc<dyn SocialArticleData>>,
	agent: Dispatcher<TwitterAgent>
}

impl HomeTimelineEndpoint {
	pub fn new(id: EndpointId) -> Self {
		Self {
			id,
			articles: Vec::new(),
			agent: TwitterAgent::dispatcher(),
		}
	}
}

impl Endpoint for HomeTimelineEndpoint {
	fn name(&self) -> String {
		"Home Timeline Endpoint".to_owned()
	}

	fn id(&self) -> &EndpointId {
		&self.id
	}

	fn add_articles(&mut self, articles: Vec<Rc<dyn SocialArticleData>>)  {
		for a in articles {
			if !self.articles.iter().any(|existing| existing.id() == a.id()) {
				self.articles.push(a);
			}
		}
		self.articles.sort_by(sort_by_id)
	}

	fn refresh(&mut self) {
		let id = self.id().clone();
		self.agent.send(Request::FetchTweets(id, "/proxy/twitter/home?count=20".to_owned()))
	}

	fn load_bottom(&mut self) {
		match self.articles.last() {
			Some(last_id) => {
				let id = self.id().clone();
				self.agent.send(Request::FetchTweets(
					id,
					format!("/proxy/twitter/home?max_id={}", &last_id.id())
				))
			}
			None => self.refresh()
		}
	}
}

pub struct ListEndpoint {
	id: EndpointId,
	username: String,
	slug: String,
	articles: Vec<Rc<dyn SocialArticleData>>,
	agent: Dispatcher<TwitterAgent>
}

impl ListEndpoint {
	pub fn new(id: EndpointId, username: String, slug: String) -> Self {
		Self {
			id,
			username,
			slug,
			articles: Vec::new(),
			agent: TwitterAgent::dispatcher(),
		}
	}
}

impl Endpoint for ListEndpoint {
	fn name(&self) -> String {
		format!("List {}/{}", &self.username, &self.slug).to_owned()
	}

	fn id(&self) -> &EndpointId {
		&self.id
	}

	fn add_articles(&mut self, articles: Vec<Rc<dyn SocialArticleData>>)  {
		for a in articles {
			if !self.articles.iter().any(|existing| existing.id() == a.id()) {
				self.articles.push(a);
			}
		}
		self.articles.sort_by(sort_by_id)
	}

	fn refresh(&mut self) {
		let id = self.id().clone();
		self.agent.send(Request::FetchTweets(
			id,
			format!("/proxy/twitter/list/{}/{}", &self.username, &self.slug)
		))
	}

	fn load_bottom(&mut self) {
		match self.articles.last() {
			Some(last_id) => {
				let id = self.id().clone();
				self.agent.send(Request::FetchTweets(
					id,
					format!("/proxy/twitter/list/{}/{}?max_id={}", &self.username, &self.slug, &last_id.id())
				))
			}
			None => self.refresh()
		}
	}
}

pub struct SingleTweetEndpoint {
	id: EndpointId,
	tweet_id: u64,
	article: Option<Rc<dyn SocialArticleData>>,
	agent: Dispatcher<TwitterAgent>,
}

impl SingleTweetEndpoint {
	pub fn new(id: EndpointId, tweet_id: u64) -> Self {
		Self {
			id,
			tweet_id,
			article: None,
			agent: TwitterAgent::dispatcher(),
		}
	}
}

impl Endpoint for SingleTweetEndpoint {
	fn name(&self) -> String {
		format!("Single Tweet {}", &self.tweet_id).to_owned()
	}

	fn id(&self) -> &EndpointId {
		&self.id
	}

	fn add_articles(&mut self, articles: Vec<Rc<dyn SocialArticleData>>)  {
		if !articles.is_empty() {
			if articles.len() > 1 {
				log::warn!("SingleTweetEndpoint received multiple articles...");
			}

			self.article = Some(articles[0].clone());
		}
	}

	fn refresh(&mut self) {
		let id = self.id().clone();
		self.agent.send(Request::FetchTweet(
			id,
			format!("/proxy/twitter/status/{}", &self.tweet_id)
		))
	}
}