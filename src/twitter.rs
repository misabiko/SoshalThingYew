use std::{rc::Rc, collections::HashSet};
use wasm_bindgen::prelude::*;
use yew::worker::*;
use yewtil::future::LinkFuture;

use crate::articles::SocialArticleData;
use crate::endpoints::{Endpoint, EndpointMsg, EndpointRequest, EndpointResponse};

#[derive(Clone, PartialEq)]
pub struct TwitterUser {
	pub username: String,
	pub name: String,
	pub avatar_url: String,
}

pub struct TweetArticleData {
	id: String,
	text: Option<String>,
	author: TwitterUser,
	media: Vec<String>
}

impl SocialArticleData for TweetArticleData {
	fn id(&self) -> String {
		self.id.clone()
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

	fn media(&self) -> Vec<String> {
		self.media.clone()
	}
}

async fn fetch_tweets(url: &str) -> Result<Vec<Rc<dyn SocialArticleData>>, JsValue> {
	let json_str = reqwest::Client::builder().build()?
		.get(format!("http://localhost:8080{}", url))
		.query(&[("rts", false), ("replies", false)])
		.send().await?
		.text().await?
		.to_string();

	let value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
	Ok(value
		.as_array().unwrap()
		.iter()
		.map(|json| Rc::new(TweetArticleData::from(json)) as Rc<dyn SocialArticleData>)
		.collect())
}

async fn fetch_tweet(id: &str) -> Result<Rc<TweetArticleData>, JsValue> {
	let json_str = reqwest::get(format!("http://localhost:8080/proxy/twitter/status/{}", &id))
		.await?
		.text()
		.await?;

	let value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
	Ok(Rc::new(TweetArticleData::from(value)))
}

impl From<&serde_json::Value> for TweetArticleData {
	fn from(json: &serde_json::Value) -> Self {
		let medias_opt = json["extended_entities"]
		.get("media")
		.and_then(|media| media.as_array());
		TweetArticleData {
			id: json["id"].as_u64().unwrap().to_string(),
			text: match json["full_text"].as_str() {
				Some(text) => Some(text),
				None => json["text"].as_str()
			}.map(String::from),
			author: TwitterUser {
				username: json["user"]["screen_name"].as_str().unwrap().to_owned(),
				name: json["user"]["name"].as_str().unwrap().to_owned(),
				avatar_url: json["user"]["profile_image_url_https"].as_str().unwrap().to_owned(),
			},
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
	subscribers: HashSet<HandlerId>,
}

pub enum AgentRequest {
	//UpdateRateLimit(RateLimit),
	EventBusMsg(String),
}

pub enum AgentOutput {
	//UpdatedRateLimit(RateLimit),
}

impl TwitterAgent {

}

impl Agent for TwitterAgent {
	type Reach = Context<Self>;
	type Message = ();
	type Input = AgentRequest;
	type Output = String;

	fn create(link: AgentLink<Self>) -> Self {
		Self {
			link,
			subscribers: HashSet::new(),
		}
	}

	fn update(&mut self, _msg: Self::Message) {}

	fn connected(&mut self, id: HandlerId) {
		self.subscribers.insert(id);
	}

	fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
		match msg {
			AgentRequest::EventBusMsg(s) => {
				for sub in self.subscribers.iter() {
					self.link.respond(*sub, s.clone());
				}
			}
		}
	}

	fn disconnected(&mut self, id: HandlerId) {
		self.subscribers.remove(&id);
	}
}

pub struct TwitterEndpoint {
	link: AgentLink<Self>,
	subscribers: HashSet<HandlerId>,
	kind: EndpointKind,
	ratelimit: RateLimit,
}

pub enum EndpointKind {
	Uninitialized,
	UserTimeline(String),
	List(String, String),
}

impl Endpoint for TwitterEndpoint {
	fn name(&self) -> String {
		match &self.kind {
			EndpointKind::Uninitialized => "Uninitialized".to_owned(),
			EndpointKind::UserTimeline(username) => format!("User Timeline {}", username),
			EndpointKind::List(username, slug) => format!("List {}/{}", username, slug),
		}
	}
}

impl Agent for TwitterEndpoint {
	type Reach = Context<Self>;
	type Message = EndpointMsg;
	type Input = EndpointRequest;
	type Output = EndpointResponse;

	fn create(link: AgentLink<Self>) -> Self {
		Self {
			link,
			subscribers: HashSet::new(),
			kind: EndpointKind::Uninitialized,
			ratelimit: RateLimit {
				limit: 1,
				remaining: 1,
				reset: 0,
			}
		}
	}

	fn update(&mut self, msg: Self::Message) {
		match msg {
			//AgentRequest::UpdateRateLimit(rateLimit) => self.ratelimit = rateLimit,
			EndpointMsg::Refreshed(tweets) => {
				for sub in self.subscribers.iter() {
					self.link.respond(*sub, EndpointResponse::NewArticles(tweets.clone()));
				}
			}
			EndpointMsg::RefreshFail(err) => {
				log::error!("Failed to fetch \"/proxy/art\"\n{:?}", err);
			}
		};
	}

	fn connected(&mut self, id: HandlerId) {
		self.subscribers.insert(id);
	}

	fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
		match msg {
			EndpointRequest::Refresh => {
				self.link.send_future(async {
					match fetch_tweets("/proxy/art").await {
						Ok(vec_tweets) => EndpointMsg::Refreshed(vec_tweets),
						Err(err) => EndpointMsg::RefreshFail(err)
					}
				});
			}
		}
	}

	fn disconnected(&mut self, id: HandlerId) {
		self.subscribers.remove(&id);
	}
}