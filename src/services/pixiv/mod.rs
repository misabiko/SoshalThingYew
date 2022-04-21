use std::cell::RefCell;
use std::rc::Rc;
use yew_agent::{Agent, AgentLink, Context, HandlerId, Dispatched, Dispatcher};
use std::collections::{HashMap, HashSet};

pub mod endpoints;
pub mod article;

use article::{PixivArticleData, PixivArticleCached};

use crate::articles::{ArticleRc, ArticleWeak};
use crate::error::RatelimitedResult;
use crate::services::{
	service,
	article_actions::{ArticleActionsAgent, ServiceActions, Request as ArticleActionsRequest},
	endpoint_agent::{EndpointAgent, Request as EndpointRequest, EndpointId, RefreshTime, EndpointConstructorCollection, EndpointConstructor},
	pixiv::endpoints::{APIPayload, FollowAPIEndpoint, FollowAPIResponse, FullPostAPI},
	storages::{ServiceStorage, get_service_storage, cache_articles},
};

#[service("Pixiv", PixivArticleData, u32)]
pub struct PixivAgent {
	link: AgentLink<Self>,
	endpoint_agent: Dispatcher<EndpointAgent>,
	actions_agent: Dispatcher<ArticleActionsAgent>,
	fetching_articles: HashSet<u32>,
}

pub enum Msg {
	FetchResponse(RatelimitedResult<Vec<ArticleRc<PixivArticleData>>>),
	EndpointFetchResponse(RefreshTime, EndpointId, RatelimitedResult<Vec<ArticleRc<PixivArticleData>>>),
	FetchData(HandlerId, ArticleWeak),
}

pub enum Request {
	AddArticles(RefreshTime, EndpointId, Vec<ArticleRc<PixivArticleData>>),
	RefreshEndpoint(EndpointId, RefreshTime),
	FetchPosts(RefreshTime, EndpointId, String),
}

impl Agent for PixivAgent {
	type Reach = Context<Self>;
	type Message = Msg;
	type Input = Request;
	type Output = ();

	fn create(link: AgentLink<Self>) -> Self {
		let mut endpoint_agent = EndpointAgent::dispatcher();
		endpoint_agent.send(EndpointRequest::InitService(
			SERVICE_INFO.name,
			EndpointConstructorCollection {
				constructors: vec![
					EndpointConstructor {
						name: "Follow API",
						param_template: vec![
							("r18", serde_json::Value::Bool(false)),
							("current_page", serde_json::Value::Number(0.into())),
						],
						callback: Rc::new(|id, params| Box::new(FollowAPIEndpoint::from_json(id, params))),
					},
				],
				user_endpoint_index: None,
			}));

		let mut actions_agent = ArticleActionsAgent::dispatcher();
		actions_agent.send(ArticleActionsRequest::Init(SERVICE_INFO.name, ServiceActions {
			like: None,
			repost: None,
			fetch_data: Some(link.callback(|(id, article)| Msg::FetchData(id, article))),
		}));

		Self {
			link,
			endpoint_agent,
			actions_agent,
			articles: HashMap::new(),
			fetching_articles: HashSet::new(),
		}
	}

	fn update(&mut self, msg: Self::Message) {
		match msg {
			Msg::EndpointFetchResponse(refresh_time, id, r) => {
				let mut valid_rc = Vec::new();
				if let Ok((articles, _)) = &r {
					for article in articles {
						let borrow = article.borrow();
						let valid_a_rc = self.articles.entry(borrow.id)
							.and_modify(|a| a.borrow_mut().update(&borrow))
							.or_insert_with(|| article.clone()).clone();

						valid_rc.push(valid_a_rc);
					}
				}
				self.endpoint_agent.send(EndpointRequest::EndpointFetchResponse(
					refresh_time,
					id,
					r.map(move |(_, ratelimit)|
						(
							valid_rc.into_iter()
								.map(|article| article as ArticleRc)
								.collect(),
							ratelimit
						)),
				));

				self.check_unfetched_articles();
			}
			Msg::FetchResponse(r) => {
				if let Ok((articles, _)) = &r {
					let mut valid_rc = Vec::new();
					for article in articles {
						let borrow = article.borrow();
						let id = borrow.id;
						let updated = self.articles.entry(id)
							.and_modify(|a| a.borrow_mut().update(&borrow))
							.or_insert_with(|| article.clone());

						valid_rc.push(Rc::downgrade(updated) as ArticleWeak);

						self.fetching_articles.remove(&id);
					}

					self.check_unfetched_articles();
					self.actions_agent.send(ArticleActionsRequest::RedrawTimelines(valid_rc));
				}
			}
			Msg::FetchData(_handler_id, article) => {
				let strong = article.upgrade().unwrap();
				let borrow = strong.borrow();

				let path = format!("https://www.pixiv.net/ajax/illust/{}", borrow.id());

				self.fetching_articles.insert(borrow.id().parse::<u32>().unwrap());
				self.link.send_future(async move {
					Msg::FetchResponse(fetch_post(&path, &get_service_storage(SERVICE_INFO.name)).await.map(|(article, _)| (vec![article], None)))
				});
			}
		}
	}

	fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
		match msg {
			Request::AddArticles(refresh_time, endpoint_id, articles) => {
				let mut valid_rc = Vec::new();
				for article in articles.into_iter() {
					let borrow = article.borrow();
					let valid_a_rc = self.articles.entry(borrow.id)
						.and_modify(|a| a.borrow_mut().update(&borrow))
						.or_insert_with(|| article.clone()).clone();

					valid_rc.push(valid_a_rc);
				}
				self.endpoint_agent.send(EndpointRequest::AddArticles(
					refresh_time,
					endpoint_id,
					valid_rc.into_iter()
						.map(|article| article as ArticleRc)
						.collect(),
				));

				self.check_unfetched_articles();
			}
			Request::RefreshEndpoint(endpoint_id, refresh_time) => self.endpoint_agent.send(EndpointRequest::RefreshEndpoint(endpoint_id, refresh_time)),
			Request::FetchPosts(refresh_time, endpoint_id, path) =>
				self.link.send_future(async move {
					Msg::EndpointFetchResponse(refresh_time, endpoint_id, fetch_posts(&path, &get_service_storage(SERVICE_INFO.name)).await)
				})
		};
	}
}

impl PixivAgent {
	fn check_unfetched_articles(&mut self) {
		let unfetched: Vec<u32> = self.articles.values().filter_map(|a| if !a.borrow().is_fully_fetched && !self.fetching_articles.contains(&a.borrow().id) {
			Some(a.borrow().id)
		} else {
			None
		}).collect();
		let count = unfetched.len();
		log::debug!("{} articles unfetched out of {}, currently fetching {}.", &count, self.articles.len(), self.fetching_articles.len());

		if count > 0 {
			if self.fetching_articles.len() < 5 {
				for id in unfetched.into_iter().take(5) {
					let path = format!("https://www.pixiv.net/ajax/illust/{}", &id);

					self.fetching_articles.insert(id);
					self.link.send_future(async move {
						Msg::FetchResponse(fetch_post(&path, &get_service_storage(SERVICE_INFO.name)).await.map(|(article, _)| (vec![article], None)))
					});
				}
			}
		} else if self.fetching_articles.is_empty() {
			self.cache_articles();
		}
	}

	fn cache_articles(&self) {
		log::debug!("Caching Pixiv articles...");

		cache_articles(SERVICE_INFO.name, self.articles.iter()
			.map(|(id, a)| (id.to_string(), serde_json::to_value(PixivArticleCached::from(&a.borrow())).unwrap()))
			.collect());
	}
}

//TODO Stop using RatelimitedResult
async fn fetch_posts(url: &str, storage: &ServiceStorage) -> RatelimitedResult<Vec<ArticleRc<PixivArticleData>>> {
	let response = reqwest::Client::builder()
		//.timeout(Duration::from_secs(10))
		.build()?
		.get(url)
		.send().await?;

	let json_str = response.text().await?.to_string();

	let response: serde_json::Value = serde_json::from_str(&json_str)?;
	let parsed: APIPayload<FollowAPIResponse> = serde_json::from_value(response.clone())?;
	if parsed.error {
		Err(parsed.message.into())
	} else {
		Ok((parsed.body.thumbnails.illust
				.iter().zip(response["body"]["thumbnails"]["illust"].as_array().unwrap())
				.map(|(a, raw_json)| PixivArticleData::from((raw_json.clone(), a, storage)))
				.map(|p| Rc::new(RefCell::new(p)))
				.collect(),
			None))
	}
}

async fn fetch_post(url: &str, storage: &ServiceStorage) -> RatelimitedResult<ArticleRc<PixivArticleData>> {
	let response = reqwest::Client::builder()
		//.timeout(Duration::from_secs(10))
		.build()?
		.get(url)
		.send().await?;

	let json_str = response.text().await?.to_string();

	let response: serde_json::Value = serde_json::from_str(&json_str)?;
	let parsed: APIPayload<FullPostAPI> = serde_json::from_value(response.clone())?;
	Ok((Rc::new(RefCell::new(PixivArticleData::from((response["body"].clone(), parsed.body, storage)))), None))
}