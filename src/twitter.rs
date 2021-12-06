use std::{rc::Rc, collections::HashSet};
use wasm_bindgen::prelude::*;
use yew::worker::*;
use yew::agent::{Dispatched, Dispatcher};
use yewtil::future::LinkFuture;

use crate::articles::SocialArticleData;
use crate::endpoints::{EndpointAgent, Endpoint, EndpointRequest};

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
	endpoints: Vec<Rc<dyn Endpoint>>,
}

pub enum AgentRequest {
	//UpdateRateLimit(RateLimit),
	EventBusMsg(String),
}

pub enum AgentOutput {
	//UpdatedRateLimit(RateLimit),
}

pub enum AgentMsg {
	Init,
}

impl Agent for TwitterAgent {
	type Reach = Context<Self>;
	type Message = AgentMsg;
	type Input = AgentRequest;
	type Output = String;

	fn create(link: AgentLink<Self>) -> Self {
		link.send_message(AgentMsg::Init);

		Self {
			link,
			subscribers: HashSet::new(),
			/*endpoints: vec![Rc::from(TwitterEndpoint {
				article: Rc::from(TweetArticleData {
					id: "1467723852239470594".to_owned(),
					text: Some("🤞".to_owned()),
					author: TwitterUser {
						username: "Banya_Bana".to_owned(),
						name: "BANYA".to_owned(),
						avatar_url: "https://pbs.twimg.com/profile_images/1467723898095824898/HCM0q8Mh_200x200.jpg".to_owned(),
					},
					media: vec!["https://pbs.twimg.com/media/FF5m5NFaUAAAOGl?format=png".to_owned()]
				})
			})]*/
			endpoints: Vec::new()
		}
	}

	fn update(&mut self, msg: Self::Message) {
		match msg {
			AgentMsg::Init => {
				EndpointAgent::dispatcher().send(EndpointRequest::AddEndpoint(self.endpoints[0].clone()));
			}
		}
	}

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

/*impl Agent for TwitterEndpoint {
	type Reach = Context<Self>;
	type Message = EndpointMsg;
	type Input = EndpointRequest;
	type Output = EndpointResponse;

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
}*/

pub struct UserTimelineEndpoint {
	username: String,
	agent: Dispatcher<TwitterAgent>
}

impl Endpoint for UserTimelineEndpoint {
	fn name(&self) -> String {
		format!("{} User Timeline Endpoint", &self.username).to_owned()
	}

	fn refresh(&self) {}
}

pub struct ListEndpoint {
	username: String,
	slug: String,
	agent: Dispatcher<TwitterAgent>
}

impl Endpoint for ListEndpoint {
	fn name(&self) -> String {
		format!("List {}/{}", &self.username, &self.slug).to_owned()
	}

	fn refresh(&self) {}
}

pub struct ArtEndpoint {
	agent: Dispatcher<TwitterAgent>
}

impl Endpoint for ArtEndpoint {
	fn name(&self) -> String {
		"Art Endpoint".to_owned()
	}

	fn refresh(&self) {
		/*self.agent.send_future(async {
			match fetch_tweets("/proxy/art").await {
				Ok(vec_tweets) => AgentRequest::Refreshed(vec_tweets),
				Err(err) => EndpointMsg::RefreshFail(err)
			}
		});*/
	}
}