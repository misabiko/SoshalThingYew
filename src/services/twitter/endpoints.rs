use std::rc::Rc;
use yew_agent::{Dispatched, Dispatcher};

use crate::articles::{ArticleData};
use crate::services::twitter::{TwitterAgent, Request as TwitterRequest};
use crate::services::endpoints::{Endpoint, EndpointId, RateLimit, RefreshTime};

pub struct UserTimelineEndpoint {
	id: EndpointId,
	username: String,
	articles: Vec<Rc<dyn ArticleData>>,
	agent: Dispatcher<TwitterAgent>,
	ratelimit: RateLimit,
}

impl UserTimelineEndpoint {
	pub fn new(id: EndpointId, username: String) -> Self {
		Self {
			id,
			username,
			articles: Vec::new(),
			agent: TwitterAgent::dispatcher(),
			ratelimit: RateLimit::default()
		}
	}

	pub fn from_json(id: EndpointId, value: serde_json::Value) -> Self {
		Self::new(id, value["username"].as_str().unwrap().to_owned())
	}
}

impl Endpoint for UserTimelineEndpoint {
	fn name(&self) -> String {
		format!("{} User Timeline Endpoint", &self.username).to_owned()
	}

	fn id(&self) -> &EndpointId {
		&self.id
	}

	fn articles(&mut self) -> &mut Vec<Rc<dyn ArticleData>> {
		&mut self.articles
	}

	fn ratelimit(&self) -> Option<&RateLimit> {
		Some(&self.ratelimit)
	}

	fn update_ratelimit(&mut self, ratelimit: RateLimit) {
		self.ratelimit = ratelimit
	}

	fn refresh(&mut self, refresh_time: RefreshTime) {
		let id = self.id().clone();
		self.agent.send(TwitterRequest::FetchTweets(
			refresh_time,
			id,
			format!("/proxy/twitter/user/{}?count=20", &self.username)
		))
	}

	fn load_bottom(&mut self, refresh_time: RefreshTime) {
		match self.articles.last() {
			Some(last_id) => {
				let id = self.id().clone();
				self.agent.send(TwitterRequest::FetchTweets(
					refresh_time,
					id,
					format!("/proxy/twitter/user/{}?max_id={}", &self.username, &last_id.id())
				))
			}
			None => self.refresh(refresh_time)
		}
	}
}

pub struct HomeTimelineEndpoint {
	id: EndpointId,
	articles: Vec<Rc<dyn ArticleData>>,
	agent: Dispatcher<TwitterAgent>,
	ratelimit: RateLimit,
}

impl HomeTimelineEndpoint {
	pub fn new(id: EndpointId) -> Self {
		Self {
			id,
			articles: Vec::new(),
			agent: TwitterAgent::dispatcher(),
			ratelimit: RateLimit::default()
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

	fn articles(&mut self) -> &mut Vec<Rc<dyn ArticleData>> {
		&mut self.articles
	}

	fn ratelimit(&self) -> Option<&RateLimit> {
		Some(&self.ratelimit)
	}

	fn update_ratelimit(&mut self, ratelimit: RateLimit) {
		self.ratelimit = ratelimit
	}

	fn refresh(&mut self, refresh_time: RefreshTime) {
		let id = self.id().clone();
		self.agent.send(TwitterRequest::FetchTweets(refresh_time, id, "/proxy/twitter/home?count=20".to_owned()))
	}

	fn load_bottom(&mut self, refresh_time: RefreshTime) {
		match self.articles.last() {
			Some(last_id) => {
				let id = self.id().clone();
				self.agent.send(TwitterRequest::FetchTweets(
					refresh_time,
					id,
					format!("/proxy/twitter/home?max_id={}", &last_id.id())
				))
			}
			None => self.refresh(refresh_time)
		}
	}
}

pub struct ListEndpoint {
	id: EndpointId,
	username: String,
	slug: String,
	articles: Vec<Rc<dyn ArticleData>>,
	agent: Dispatcher<TwitterAgent>,
	ratelimit: RateLimit,
}

impl ListEndpoint {
	pub fn new(id: EndpointId, username: String, slug: String) -> Self {
		Self {
			id,
			username,
			slug,
			articles: Vec::new(),
			agent: TwitterAgent::dispatcher(),
			ratelimit: RateLimit::default()
		}
	}

	pub fn from_json(id: EndpointId, value: serde_json::Value) -> Self {
		Self::new(
			id,
			value["username"].as_str().unwrap().to_owned(),
			value["slug"].as_str().unwrap().to_owned(),
		)
	}
}

impl Endpoint for ListEndpoint {
	fn name(&self) -> String {
		format!("List {}/{}", &self.username, &self.slug).to_owned()
	}

	fn id(&self) -> &EndpointId {
		&self.id
	}

	fn articles(&mut self) -> &mut Vec<Rc<dyn ArticleData>> {
		&mut self.articles
	}

	fn ratelimit(&self) -> Option<&RateLimit> {
		Some(&self.ratelimit)
	}

	fn update_ratelimit(&mut self, ratelimit: RateLimit) {
		self.ratelimit = ratelimit
	}

	fn refresh(&mut self, refresh_time: RefreshTime) {
		let id = self.id().clone();
		self.agent.send(TwitterRequest::FetchTweets(
			refresh_time,
			id,
			format!("/proxy/twitter/list/{}/{}", &self.username, &self.slug)
		))
	}

	fn load_bottom(&mut self, refresh_time: RefreshTime) {
		match self.articles.last() {
			Some(last_id) => {
				let id = self.id().clone();
				self.agent.send(TwitterRequest::FetchTweets(
					refresh_time,
					id,
					format!("/proxy/twitter/list/{}/{}?max_id={}", &self.username, &self.slug, &last_id.id())
				))
			}
			None => self.refresh(refresh_time)
		}
	}
}

pub struct SingleTweetEndpoint {
	id: EndpointId,
	tweet_id: u64,
	articles: Vec<Rc<dyn ArticleData>>,
	agent: Dispatcher<TwitterAgent>,
	ratelimit: RateLimit,
}

impl SingleTweetEndpoint {
	pub fn new(id: EndpointId, tweet_id: u64) -> Self {
		Self {
			id,
			tweet_id,
			articles: Vec::new(),
			agent: TwitterAgent::dispatcher(),
			ratelimit: RateLimit::default(),
		}
	}

	pub fn from_json(id: EndpointId, value: serde_json::Value) -> Self {
		Self::new(
			id,
			value["id"].as_u64().unwrap(),
		)
	}
}

impl Endpoint for SingleTweetEndpoint {
	fn name(&self) -> String {
		format!("Single Tweet {}", &self.tweet_id).to_owned()
	}

	fn id(&self) -> &EndpointId {
		&self.id
	}

	fn articles(&mut self) -> &mut Vec<Rc<dyn ArticleData>> {
		&mut self.articles
	}

	fn ratelimit(&self) -> Option<&RateLimit> {
		Some(&self.ratelimit)
	}

	fn update_ratelimit(&mut self, ratelimit: RateLimit) {
		self.ratelimit = ratelimit
	}

	fn refresh(&mut self, refresh_time: RefreshTime) {
		let id = self.id().clone();
		self.agent.send(TwitterRequest::FetchTweet(
			refresh_time,
			id,
			format!("/proxy/twitter/status/{}", &self.tweet_id)
		))
	}
}