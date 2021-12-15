use std::rc::Rc;
use yew_agent::{Dispatched, Dispatcher};

use crate::articles::{ArticleData};
use crate::services::twitter::{TwitterAgent, Request as TwitterRequest};
use crate::services::endpoints::{Endpoint, EndpointId};

pub struct UserTimelineEndpoint {
	id: EndpointId,
	username: String,
	articles: Vec<Rc<dyn ArticleData>>,
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

	fn articles(&mut self) -> &mut Vec<Rc<dyn ArticleData>> {
		&mut self.articles
	}

	fn refresh(&mut self) {
		let id = self.id().clone();
		self.agent.send(TwitterRequest::FetchTweets(
			id,
			format!("/proxy/twitter/user/{}", &self.username)
		))
	}

	fn load_bottom(&mut self) {
		match self.articles.last() {
			Some(last_id) => {
				let id = self.id().clone();
				self.agent.send(TwitterRequest::FetchTweets(
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
	articles: Vec<Rc<dyn ArticleData>>,
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

	fn articles(&mut self) -> &mut Vec<Rc<dyn ArticleData>> {
		&mut self.articles
	}

	fn refresh(&mut self) {
		let id = self.id().clone();
		self.agent.send(TwitterRequest::FetchTweets(id, "/proxy/twitter/home?count=20".to_owned()))
	}

	fn load_bottom(&mut self) {
		match self.articles.last() {
			Some(last_id) => {
				let id = self.id().clone();
				self.agent.send(TwitterRequest::FetchTweets(
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
	articles: Vec<Rc<dyn ArticleData>>,
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

	fn articles(&mut self) -> &mut Vec<Rc<dyn ArticleData>> {
		&mut self.articles
	}

	fn refresh(&mut self) {
		let id = self.id().clone();
		self.agent.send(TwitterRequest::FetchTweets(
			id,
			format!("/proxy/twitter/list/{}/{}", &self.username, &self.slug)
		))
	}

	fn load_bottom(&mut self) {
		match self.articles.last() {
			Some(last_id) => {
				let id = self.id().clone();
				self.agent.send(TwitterRequest::FetchTweets(
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
	articles: Vec<Rc<dyn ArticleData>>,
	agent: Dispatcher<TwitterAgent>,
}

impl SingleTweetEndpoint {
	pub fn new(id: EndpointId, tweet_id: u64) -> Self {
		Self {
			id,
			tweet_id,
			articles: Vec::new(),
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

	fn articles(&mut self) -> &mut Vec<Rc<dyn ArticleData>> {
		&mut self.articles
	}

	fn refresh(&mut self) {
		let id = self.id().clone();
		self.agent.send(TwitterRequest::FetchTweet(
			id,
			format!("/proxy/twitter/status/{}", &self.tweet_id)
		))
	}
}