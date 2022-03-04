use reqwest::Url;
use yew_agent::{Dispatched, Dispatcher};

use super::{TwitterAgent, Request as TwitterRequest, SERVICE_INFO};
use crate::articles::ArticleWeak;
use crate::base_url;
use crate::services::{Endpoint, EndpointSerialized, RateLimit};
use crate::services::endpoint_agent::{EndpointId, RefreshTime};

pub struct UserTimelineEndpoint {
	id: EndpointId,
	username: String,
	include_retweets: bool,
	include_replies: bool,
	articles: Vec<ArticleWeak>,
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
		Self::new(id, value["username"].as_str().unwrap().to_owned(), value["include_retweets"].as_bool().unwrap().to_owned(), value["include_replies"].as_bool().unwrap().to_owned())
	}
}

impl Endpoint for UserTimelineEndpoint {
	fn name(&self) -> String {
		format!("{} User Timeline Endpoint", &self.username).to_owned()
	}

	fn id(&self) -> &EndpointId {
		&self.id
	}

	fn articles(&mut self) -> &mut Vec<ArticleWeak> {
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
		self.agent.send(TwitterRequest::FetchTweets(
			refresh_time,
			self.id,
			Url::parse(&format!("{}/proxy/twitter/user/{}?replies={}&rts={}&count=20", base_url(), self.username, &self.include_replies, &self.include_retweets)).unwrap()
		))
	}

	fn load_bottom(&mut self, refresh_time: RefreshTime) {
		match self.articles.last() {
			Some(last_id) => {
				self.agent.send(TwitterRequest::FetchTweets(
					refresh_time,
					self.id,
					Url::parse(&format!("{}/proxy/twitter/user/{}?replies={}&rts={}&max_id={}", base_url(), &self.username, &self.include_replies, &self.include_retweets, &last_id.upgrade().unwrap().borrow().id())).unwrap()
				))
			}
			None => self.refresh(refresh_time)
		}
	}

	fn eq_storage(&self, storage: &EndpointSerialized) -> bool {
		storage.service == SERVICE_INFO.name &&
		storage.endpoint_type == 1 &&
		storage.params["username"]
			.as_str()
			.map(|u| u == self.username)
			.unwrap_or_default()
	}
}

pub struct HomeTimelineEndpoint {
	id: EndpointId,
	articles: Vec<ArticleWeak>,
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

	fn articles(&mut self) -> &mut Vec<ArticleWeak> {
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
		self.agent.send(TwitterRequest::FetchTweets(
			refresh_time,
			self.id,
			Url::parse(&format!("{}/proxy/twitter/home?count=20", base_url())).unwrap())
		)
	}

	fn load_bottom(&mut self, refresh_time: RefreshTime) {
		match self.articles.last() {
			Some(last_id) => {
				self.agent.send(TwitterRequest::FetchTweets(
					refresh_time,
					self.id,
					Url::parse(&format!("{}/proxy/twitter/home?max_id={}", base_url(), &last_id.upgrade().unwrap().borrow().id())).unwrap()
				))
			}
			None => self.refresh(refresh_time)
		}
	}

	fn eq_storage(&self, storage: &EndpointSerialized) -> bool {
		storage.service == SERVICE_INFO.name &&
		storage.endpoint_type == 0
	}
}

pub struct ListEndpoint {
	id: EndpointId,
	username: String,
	slug: String,
	articles: Vec<ArticleWeak>,
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

	fn articles(&mut self) -> &mut Vec<ArticleWeak> {
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
		self.agent.send(TwitterRequest::FetchTweets(
			refresh_time,
			self.id,
			Url::parse(&format!("{}/proxy/twitter/list/{}/{}", base_url(), &self.username, &self.slug)).unwrap()
		))
	}

	fn load_bottom(&mut self, refresh_time: RefreshTime) {
		match self.articles.last() {
			Some(last_id) => {
				self.agent.send(TwitterRequest::FetchTweets(
					refresh_time,
					self.id,
					Url::parse(&format!("{}/proxy/twitter/list/{}/{}?max_id={}", base_url(), &self.username, &self.slug, &last_id.upgrade().unwrap().borrow().id())).unwrap()
				))
			}
			None => self.refresh(refresh_time)
		}
	}

	fn eq_storage(&self, storage: &EndpointSerialized) -> bool {
		storage.service == SERVICE_INFO.name &&
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

pub struct LikesEndpoint {
	id: EndpointId,
	username: String,
	articles: Vec<ArticleWeak>,
	agent: Dispatcher<TwitterAgent>,
	ratelimit: RateLimit,
}

impl LikesEndpoint {
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
		Self::new(
			id,
			value["username"].as_str().unwrap().to_owned(),
		)
	}
}

impl Endpoint for LikesEndpoint {
	fn name(&self) -> String {
		format!("Liked by {}", &self.username).to_owned()
	}

	fn id(&self) -> &EndpointId {
		&self.id
	}

	fn articles(&mut self) -> &mut Vec<ArticleWeak> {
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
		self.agent.send(TwitterRequest::FetchTweets(
			refresh_time,
			self.id,
			Url::parse(&format!("{}/proxy/twitter/likes/{}?count=20", base_url(), &self.username)).unwrap()
		))
	}

	fn load_bottom(&mut self, refresh_time: RefreshTime) {
		match self.articles.last() {
			Some(last_id) => {
				self.agent.send(TwitterRequest::FetchTweets(
					refresh_time,
					self.id,
					Url::parse(&format!("{}/proxy/twitter/likes/{}?max_id={}", base_url(), &self.username, &last_id.upgrade().unwrap().borrow().id())).unwrap()
				))
			}
			None => self.refresh(refresh_time)
		}
	}

	fn eq_storage(&self, storage: &EndpointSerialized) -> bool {
		storage.service == SERVICE_INFO.name &&
			storage.endpoint_type == 3 &&
			storage.params["username"]
				.as_str()
				.map(|u| u == self.username)
				.unwrap_or_default()
	}
}

pub struct SingleTweetEndpoint {
	id: EndpointId,
	tweet_id: u64,
	articles: Vec<ArticleWeak>,
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
			value["id"].as_str().unwrap().parse::<u64>().unwrap(),
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

	fn articles(&mut self) -> &mut Vec<ArticleWeak> {
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
		self.agent.send(TwitterRequest::FetchTweet(
			refresh_time,
			self.id,
			Url::parse(&format!("{}/proxy/twitter/status/{}", base_url(), &self.tweet_id)).unwrap()
		))
	}

	fn eq_storage(&self, storage: &EndpointSerialized) -> bool {
		storage.service == SERVICE_INFO.name &&
		storage.endpoint_type == 4 &&
		storage.params["id"]
			.as_u64()
			.map(|id| id == self.tweet_id)
			.unwrap_or_default()
	}
}

pub struct SearchEndpoint {
	id: EndpointId,
	query: String,
	articles: Vec<ArticleWeak>,
	agent: Dispatcher<TwitterAgent>,
	ratelimit: RateLimit,
}

impl SearchEndpoint {
	pub fn new(id: EndpointId, query: String) -> Self {
		Self {
			id,
			query,
			articles: Vec::new(),
			agent: TwitterAgent::dispatcher(),
			ratelimit: RateLimit::default()
		}
	}

	pub fn from_json(id: EndpointId, value: serde_json::Value) -> Self {
		Self::new(
			id,
			value["query"].as_str().unwrap().to_owned(),
		)
	}
}

impl Endpoint for SearchEndpoint {
	fn name(&self) -> String {
		format!("Search \"{}\"", &self.query).to_owned()
	}

	fn id(&self) -> &EndpointId {
		&self.id
	}

	fn articles(&mut self) -> &mut Vec<ArticleWeak> {
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
		let mut url = Url::parse(&format!("{}/proxy/twitter/search", base_url())).unwrap();
		url.set_query(Some(&format!("query={}", self.query)));
		self.agent.send(TwitterRequest::FetchTweets(refresh_time, self.id, url))
	}

	fn load_bottom(&mut self, refresh_time: RefreshTime) {
		match self.articles.last() {
			Some(last_id) => {
				let mut url = Url::parse(&format!("{}/proxy/twitter/search", base_url())).unwrap();
				url.query_pairs_mut()
					.append_pair("query", self.query.as_str())
					.append_pair("max_id", &last_id.upgrade().unwrap().borrow().id());
				self.agent.send(TwitterRequest::FetchTweets(refresh_time, self.id, url))
			}
			None => self.refresh(refresh_time)
		}
	}

	fn eq_storage(&self, storage: &EndpointSerialized) -> bool {
		storage.service == SERVICE_INFO.name &&
			storage.endpoint_type == 4 &&
			storage.params["query"]
				.as_str()
				.map(|s| s == self.query)
				.unwrap_or_default()
	}
}