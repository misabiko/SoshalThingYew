use std::rc::Weak;
use std::cell::RefCell;
use yew_agent::{Dispatched, Dispatcher};

use super::{TwitterAgent, Request as TwitterRequest};
use crate::articles::{ArticleData};
use crate::services::{Endpoint, EndpointStorage, RateLimit};
use crate::services::endpoint_agent::{EndpointId, RefreshTime};

pub struct UserTimelineEndpoint {
	id: EndpointId,
	username: String,
	include_retweets: bool,
	include_replies: bool,
	articles: Vec<Weak<RefCell<dyn ArticleData>>>,
	agent: Dispatcher<TwitterAgent>,
	ratelimit: RateLimit,
}

impl UserTimelineEndpoint {
	pub fn new(id: EndpointId, username: String, include_retweets: bool, include_replies: bool) -> Self {
		Self {
			id,
			username,
			include_retweets,
			include_replies,
			articles: Vec::new(),
			agent: TwitterAgent::dispatcher(),
			ratelimit: RateLimit::default()
		}
	}

	pub fn from_json(id: EndpointId, value: serde_json::Value) -> Self {
		Self::new(id, value["username"].as_str().unwrap().to_owned(), false, false)
	}
}

impl Endpoint for UserTimelineEndpoint {
	fn name(&self) -> String {
		format!("{} User Timeline Endpoint", &self.username).to_owned()
	}

	fn id(&self) -> &EndpointId {
		&self.id
	}

	fn articles(&mut self) -> &mut Vec<Weak<RefCell<dyn ArticleData>>> {
		&mut self.articles
	}

	fn ratelimit(&self) -> Option<&RateLimit> {
		Some(&self.ratelimit)
	}

	fn get_mut_ratelimit(&mut self) -> Option<&mut RateLimit> {
		Some(&mut self.ratelimit)
	}

	fn update_ratelimit(&mut self, ratelimit: RateLimit) {
		self.ratelimit = ratelimit
	}
	fn refresh(&mut self, refresh_time: RefreshTime) {
		let id = self.id().clone();
		self.agent.send(TwitterRequest::FetchTweets(
			refresh_time,
			id,
			format!("/proxy/twitter/user/{}?replies={:?}&rts={:?}&count=20", self.username, &self.include_replies, &self.include_retweets)
		))
	}

	fn load_bottom(&mut self, refresh_time: RefreshTime) {
		match self.articles.last() {
			Some(last_id) => {
				let id = self.id().clone();
				self.agent.send(TwitterRequest::FetchTweets(
					refresh_time,
					id,
					format!("/proxy/twitter/user/{}?replies={:?}&rts={:?}&max_id={}", &self.username, &self.include_replies, &self.include_retweets, &last_id.upgrade().unwrap().borrow().id())
				))
			}
			None => self.refresh(refresh_time)
		}
	}

	fn eq_storage(&self, storage: &EndpointStorage) -> bool {
		storage.service == "Twitter" &&
		storage.endpoint_type == 1 &&
		storage.params["username"]
			.as_str()
			.map(|u| u == self.username)
			.unwrap_or_default()
	}
}

pub struct HomeTimelineEndpoint {
	id: EndpointId,
	articles: Vec<Weak<RefCell<dyn ArticleData>>>,
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

	fn articles(&mut self) -> &mut Vec<Weak<RefCell<dyn ArticleData>>> {
		&mut self.articles
	}

	fn ratelimit(&self) -> Option<&RateLimit> {
		Some(&self.ratelimit)
	}

	fn get_mut_ratelimit(&mut self) -> Option<&mut RateLimit> {
		Some(&mut self.ratelimit)
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
					format!("/proxy/twitter/home?max_id={}", &last_id.upgrade().unwrap().borrow().id())
				))
			}
			None => self.refresh(refresh_time)
		}
	}

	fn eq_storage(&self, storage: &EndpointStorage) -> bool {
		storage.service == "Twitter" &&
		storage.endpoint_type == 0
	}
}

pub struct ListEndpoint {
	id: EndpointId,
	username: String,
	slug: String,
	articles: Vec<Weak<RefCell<dyn ArticleData>>>,
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

	fn articles(&mut self) -> &mut Vec<Weak<RefCell<dyn ArticleData>>> {
		&mut self.articles
	}

	fn ratelimit(&self) -> Option<&RateLimit> {
		Some(&self.ratelimit)
	}

	fn get_mut_ratelimit(&mut self) -> Option<&mut RateLimit> {
		Some(&mut self.ratelimit)
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
					format!("/proxy/twitter/list/{}/{}?max_id={}", &self.username, &self.slug, &last_id.upgrade().unwrap().borrow().id())
				))
			}
			None => self.refresh(refresh_time)
		}
	}

	fn eq_storage(&self, storage: &EndpointStorage) -> bool {
		storage.service == "Twitter" &&
		storage.endpoint_type == 2 &&
		storage.params["username"]
			.as_str()
			.map(|u| u == self.username)
			.unwrap_or_default() &&
		storage.params["slug"]
			.as_str()
			.map(|s| s == self.slug)
			.unwrap_or_default()
	}
}

pub struct SingleTweetEndpoint {
	id: EndpointId,
	tweet_id: u64,
	articles: Vec<Weak<RefCell<dyn ArticleData>>>,
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

	fn articles(&mut self) -> &mut Vec<Weak<RefCell<dyn ArticleData>>> {
		&mut self.articles
	}

	fn ratelimit(&self) -> Option<&RateLimit> {
		Some(&self.ratelimit)
	}

	fn get_mut_ratelimit(&mut self) -> Option<&mut RateLimit> {
		Some(&mut self.ratelimit)
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

	fn eq_storage(&self, storage: &EndpointStorage) -> bool {
		storage.service == "Twitter" &&
		storage.endpoint_type == 3 &&
		storage.params["id"]
			.as_u64()
			.map(|id| id == self.tweet_id)
			.unwrap_or_default()
	}
}