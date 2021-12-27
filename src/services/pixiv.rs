use std::rc::{Rc, Weak};
use std::cell::{RefCell, Ref};
use yew_agent::{Agent, AgentLink, Context, HandlerId, Dispatched, Dispatcher};
use js_sys::Date;
use std::collections::{HashMap, HashSet};
use gloo_timers::callback::Timeout;
use gloo_storage::Storage;
use serde::{Serialize, Deserialize};

use crate::articles::{ArticleData, ArticleMedia};
use crate::error::FetchResult;
use crate::services::{Endpoint, EndpointSerialized};
use crate::services::article_actions::{ArticleActionsAgent, ServiceActions, Request as ArticleActionsRequest};
use crate::services::endpoint_agent::{EndpointAgent, Request as EndpointRequest, EndpointId, RefreshTime, EndpointConstructors};
use crate::services::storages::{SessionStorageService, SoshalSessionStorage, get_service_session};

pub struct PixivArticleData {
	id: u32,
	src: String,
	title: String,
	author_name: String,
	author_id: u32,
	author_avatar_url: String,
	marked_as_read: bool,
	hidden: bool,
	is_fully_fetched: bool,
	raw_json: serde_json::Value,
}

impl ArticleData for PixivArticleData {
	fn service(&self) -> &'static str {
		"Pixiv"
	}
	fn id(&self) -> String {
		self.id.clone().to_string()
	}
	fn creation_time(&self) -> Date {
		js_sys::Date::new_0()
	}
	fn text(&self) -> String {
		self.title.clone()
	}
	fn author_username(&self) -> String {
		self.author_id.clone().to_string()
	}
	fn author_name(&self) -> String {
		self.author_name.clone()
	}
	fn author_avatar_url(&self) -> String {
		self.author_avatar_url.clone()
	}
	fn author_url(&self) -> String {
		format!("https://www.pixiv.net/en/users/{}", &self.author_id)
	}

	fn media(&self) -> Vec<ArticleMedia> {
		//TODO Pixiv image ratio
		vec![ArticleMedia::Image(self.src.clone(), 1.0)]
	}

	fn json(&self) -> serde_json::Value {
		self.raw_json.clone()
	}

	fn url(&self) -> String {
		format!("https://www.pixiv.net/en/artworks/{}", &self.id)
	}

	fn update(&mut self, new: &Ref<dyn ArticleData>) {
		self.src = match new.media().first() {
			Some(ArticleMedia::Image(src, _ratio)) => src.clone(),
			_ => "".to_owned(),
		};
		self.title = new.text();
		self.is_fully_fetched = self.is_fully_fetched || *new.is_fully_fetched();
		match new.json() {
			serde_json::Value::Null => {}
			new_json => self.raw_json = new_json,
		};
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

	fn is_fully_fetched(&self) -> &bool { &self.is_fully_fetched }
}

impl From<(serde_json::Value, &FullPostAPI, &SessionStorageService)> for PixivArticleData {
	fn from((raw_json, data, storage): (serde_json::Value, &FullPostAPI, &SessionStorageService)) -> Self {
		let cached: Option<PixivArticleCached> = storage.cached_articles.get(&data.id)
			.and_then(|json| serde_json::from_value(json.clone()).ok());
		let author_avatar_url = match cached {
			Some(PixivArticleCached { author_avatar_url, .. }) => author_avatar_url,
			None => "".to_owned(),
		};

		PixivArticleData {
			id: data.id.parse::<u32>().unwrap(),
			title: data.title.clone(),
			src: data.urls.original.clone(),
			author_name: data.user_name.clone(),
			author_id: data.user_id.parse::<u32>().unwrap(),
			author_avatar_url,
			marked_as_read: false,
			hidden: false,
			is_fully_fetched: true,
			raw_json,
		}
	}
}

impl From<(serde_json::Value, FullPostAPI, &SessionStorageService)> for PixivArticleData {
	fn from((raw_json, data, storage): (serde_json::Value, FullPostAPI, &SessionStorageService)) -> Self {
		PixivArticleData::from((raw_json, &data, storage))
	}
}

impl From<(serde_json::Value, &FollowAPIIllust, &SessionStorageService)> for PixivArticleData {
	fn from((raw_json, data, storage): (serde_json::Value, &FollowAPIIllust, &SessionStorageService)) -> Self {
		let cached: Option<PixivArticleCached> = storage.cached_articles.get(&data.id)
			.and_then(|json| serde_json::from_value(json.clone()).ok());
		let (src, is_fully_fetched) = match cached {
			Some(PixivArticleCached { src, .. }) => (src, true),
			None => (data.url.clone(), false)
		};

		PixivArticleData {
			id: data.id.parse::<u32>().unwrap(),
			title: data.title.clone(),
			src,
			author_name: data.user_name.clone(),
			author_id: data.user_id.parse::<u32>().unwrap(),
			author_avatar_url: data.profile_image_url.clone(),
			marked_as_read: false,
			hidden: false,
			is_fully_fetched,
			raw_json,
		}
	}
}

#[derive(Serialize, Deserialize, Debug)]
struct PixivArticleCached {
	id: u32,
	src: String,
	author_avatar_url: String,
}

impl From<&Ref<'_, PixivArticleData>> for PixivArticleCached {
	fn from(article: &Ref<'_, PixivArticleData>) -> Self {
		Self {
			id: article.id.clone(),
			src: article.src.clone(),
			author_avatar_url: article.author_avatar_url.clone(),
		}
	}
}

#[derive(Deserialize)]
struct APIPayload<T> {
	error: bool,
	message: String,
	body: T,
}

#[derive(Serialize, Deserialize)]
struct FullPostAPI {
	id: String,
	title: String,
	urls: FullPostAPIURLs,
	#[serde(rename = "userAccount")]
	user_account: String,
	#[serde(rename = "userName")]
	user_name: String,
	#[serde(rename = "userId")]
	user_id: String,
}

#[derive(Serialize, Deserialize)]
struct FullPostAPIURLs {
	mini: String,
	thumb: String,
	small: String,
	regular: String,
	original: String,
}

#[derive(Deserialize)]
struct FollowAPIResponse {
	page: FollowAPIPage,
	thumbnails: FollowAPIThumbnails,
}

#[derive(Deserialize)]
struct FollowAPIPage {
	ids: Vec<u32>,
}

#[derive(Deserialize)]
struct FollowAPIThumbnails {
	illust: Vec<FollowAPIIllust>,
}

#[derive(Serialize, Deserialize)]
struct FollowAPIIllust {
	id: String,
	title: String,
	url: String,
	#[serde(rename = "userId")]
	user_id: String,
	#[serde(rename = "userName")]
	user_name: String,
	#[serde(rename = "profileImageUrl")]
	profile_image_url: String,
}

pub struct PixivAgent {
	link: AgentLink<Self>,
	endpoint_agent: Dispatcher<EndpointAgent>,
	actions_agent: Dispatcher<ArticleActionsAgent>,
	articles: HashMap<u32, Rc<RefCell<PixivArticleData>>>,
	fetching_articles: HashSet<u32>,
}

pub enum Msg {
	FetchResponse(FetchResult<Vec<Rc<RefCell<PixivArticleData>>>>),
	EndpointFetchResponse(RefreshTime, EndpointId, FetchResult<Vec<Rc<RefCell<PixivArticleData>>>>),
	FetchData(HandlerId, Weak<RefCell<dyn ArticleData>>),
}

pub enum Request {
	AddArticles(RefreshTime, EndpointId, Vec<Rc<RefCell<PixivArticleData>>>),
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
			"Pixiv".to_owned(),
			EndpointConstructors {
				endpoint_types: vec![],
				user_endpoint: None,
			}));

		let mut actions_agent = ArticleActionsAgent::dispatcher();
		actions_agent.send(ArticleActionsRequest::Init("Pixiv", ServiceActions {
			like: None,
			repost: None,
			mark_as_read: None,
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
							.and_modify(|a| a.borrow_mut().update(&(borrow as Ref<dyn ArticleData>)))
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
								.map(|article| article as Rc<RefCell<dyn ArticleData>>)
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
							.and_modify(|a| a.borrow_mut().update(&(borrow as Ref<dyn ArticleData>)))
							.or_insert_with(|| article.clone());

						valid_rc.push(Rc::downgrade(updated) as Weak<RefCell<dyn ArticleData>>);

						self.fetching_articles.remove(&id);
					}

					self.check_unfetched_articles();
					self.actions_agent.send(ArticleActionsRequest::Callback(valid_rc));
				}
			}
			Msg::FetchData(_handler_id, article) => {
				let strong = article.upgrade().unwrap();
				let borrow = strong.borrow();

				let path = format!("https://www.pixiv.net/ajax/illust/{}", borrow.id());

				self.fetching_articles.insert(borrow.id().parse::<u32>().unwrap());
				self.link.send_future(async move {
					Msg::FetchResponse(fetch_post(&path, &get_service_session("Pixiv")).await.map(|(article, _)| (vec![article], None)))
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
						.and_modify(|a| a.borrow_mut().update(&(borrow as Ref<dyn ArticleData>)))
						.or_insert_with(|| article.clone()).clone();

					valid_rc.push(valid_a_rc);
				}
				self.endpoint_agent.send(EndpointRequest::AddArticles(
					refresh_time,
					endpoint_id,
					valid_rc.into_iter()
						.map(|article| article as Rc<RefCell<dyn ArticleData>>)
						.collect(),
				));

				self.check_unfetched_articles();
			}
			Request::RefreshEndpoint(endpoint_id, refresh_time) => self.endpoint_agent.send(EndpointRequest::RefreshEndpoint(endpoint_id, refresh_time)),
			Request::FetchPosts(refresh_time, endpoint_id, path) =>
				self.link.send_future(async move {
					Msg::EndpointFetchResponse(refresh_time, endpoint_id, fetch_posts(&path, &get_service_session("Pixiv")).await)
				})
		};
	}
}

impl PixivAgent {
	fn check_unfetched_articles(&mut self) {
		let unfetched: Vec<u32> = self.articles.values().filter_map(|a| if !a.borrow().is_fully_fetched && !self.fetching_articles.contains(&a.borrow().id) {
			Some(a.borrow().id.clone())
		} else {
			None
		}).collect();
		let count = unfetched.len();
		log::debug!("{} articles unfetched out of {}, currently fetching {}.", &count, self.articles.len(), self.fetching_articles.len());

		if count > 0 {
			if self.fetching_articles.len() < 5 {
				for id in unfetched.into_iter().take(5) {
					let path = format!("https://www.pixiv.net/ajax/illust/{}", &id);

					self.fetching_articles.insert(id.clone());
					self.link.send_future(async move {
						Msg::FetchResponse(fetch_post(&path, &get_service_session("Pixiv")).await.map(|(article, _)| (vec![article], None)))
					});
				}
			}
		} else if self.fetching_articles.is_empty() {
			self.cache_articles();
		}
	}

	fn cache_articles(&self) {
		log::debug!("Caching Pixiv articles...");

		let session_storage: SoshalSessionStorage = match gloo_storage::SessionStorage::get("SoshalThingYew") {
			Ok(storage) => {
				let mut session_storage: SoshalSessionStorage = storage;
				let mut service = match session_storage.services.get_mut("Pixiv") {
					Some(service) => service,
					None => {
						let service = SessionStorageService {
							articles_marked_as_read: HashSet::new(),
							cached_articles: HashMap::new(),
						};
						session_storage.services.insert("Pixiv".to_owned(), service);
						session_storage.services.get_mut("Pixiv").unwrap()
					}
				};

				for (id, article) in &self.articles {
					service.cached_articles.insert(id.to_string(), serde_json::to_value(PixivArticleCached::from(&article.borrow())).unwrap());
				}

				session_storage
			}
			Err(_err) => {
				SoshalSessionStorage {
					services: HashMap::from([
						("Pixiv".to_owned(), SessionStorageService {
							articles_marked_as_read: HashSet::new(),
							cached_articles: self.articles.iter()
								.map(|(id, a)| (id.to_string(), serde_json::to_value(PixivArticleCached::from(&a.borrow())).unwrap()))
								.collect(),
						})
					])
				}
			}
		};

		gloo_storage::SessionStorage::set("SoshalThingYew", &session_storage)
			.expect("couldn't write session storage");
	}
}

fn parse_article(element: web_sys::Element, storage: &SessionStorageService) -> Option<Rc<RefCell<PixivArticleData>>> {
	let anchors = element.get_elements_by_tag_name("a");
	let (id, id_str) = match anchors.get_with_index(0) {
		Some(a) => match a.get_attribute("data-gtm-value") {
			Some(id) => match id.parse::<u32>().ok().zip(Some(id)) {
				Some((id, id_str)) => (id, id_str),
				None => return None,
			},
			None => return None
		},
		None => return None,
	};
	let title = match anchors.get_with_index(1) {
		Some(a) => match a.text_content() {
			Some(title) => title,
			None => return None
		},
		None => return None,
	};
	let (author_id, author_name) = match anchors.get_with_index(3) {
		Some(a) => (match a.get_attribute("data-gtm-value") {
			Some(id) => match id.parse::<u32>() {
				Ok(id) => id,
				Err(_) => return None,
			},
			None => return None
		}, match a.text_content() {
			Some(title) => title,
			None => return None
		}),
		None => return None,
	};

	let imgs = element.get_elements_by_tag_name("img");

	let author_avatar_url = match imgs.get_with_index(1) {
		Some(img) => match img.get_attribute("src") {
			Some(src) => src,
			None => return None,
		}
		None => return None,
	};

	let cached: Option<serde_json::Result<PixivArticleCached>> = storage.cached_articles.get(&id_str)
		.map(|json| serde_json::from_value(json.clone()));
	let (src, is_fully_fetched) = match cached {
		Some(Ok(PixivArticleCached { src, .. })) => (src, true),
		Some(Err(_err)) => {
			let src = match imgs.get_with_index(0) {
				Some(img) => match img.get_attribute("src") {
					Some(src) => src,
					None => return None,
				}
				None => return None,
			};
			(src, false)
		}
		None => {
			let src = match imgs.get_with_index(0) {
				Some(img) => match img.get_attribute("src") {
					Some(src) => src,
					None => return None,
				}
				None => return None,
			};
			(src, false)
		}
	};

	Some(Rc::new(RefCell::new(PixivArticleData {
		id,
		src,
		author_avatar_url,
		title,
		author_id,
		author_name,
		marked_as_read: false,
		hidden: false,
		is_fully_fetched,
		raw_json: serde_json::Value::Null,
	})))
}

pub struct FollowPageEndpoint {
	id: EndpointId,
	articles: Vec<Weak<RefCell<dyn ArticleData>>>,
	agent: Dispatcher<PixivAgent>,
	timeout: Option<Timeout>,
}

impl FollowPageEndpoint {
	pub fn new(id: EndpointId) -> Self {
		Self {
			id,
			articles: Vec::new(),
			agent: PixivAgent::dispatcher(),
			timeout: None,
		}
	}
}

impl Endpoint for FollowPageEndpoint {
	fn name(&self) -> String {
		"Follow Page Endpoint".to_owned()
	}

	fn id(&self) -> &EndpointId {
		&self.id
	}

	fn articles(&mut self) -> &mut Vec<Weak<RefCell<dyn ArticleData>>> {
		&mut self.articles
	}

	fn refresh(&mut self, refresh_time: RefreshTime) {
		let mut articles = Vec::new();
		let posts_selector = gloo_utils::document()
			.query_selector(".sc-9y4be5-1.jtUPOE");
		match posts_selector {
			Ok(None) => {
				if self.timeout.is_none() {
					let mut agent = PixivAgent::dispatcher();
					let id = self.id.clone();
					let timeout = Some(Timeout::new(1_000, move || agent.send(Request::RefreshEndpoint(id, refresh_time))));
					self.timeout = timeout;
				}
			}
			Ok(Some(posts)) => {
				let children = posts.children();
				log::debug!("Found {} posts.", children.length());
				let storage = get_service_session("Pixiv");
				for i in 0..children.length() {
					if let Some(article) = children.get_with_index(i).and_then(|a| parse_article(a, &storage)) {
						articles.push(article);
					}
				}

				let id = self.id().clone();
				self.agent.send(Request::AddArticles(refresh_time, id, articles));
				self.timeout = None;
			}
			Err(err) => log::error!("Failed to use query_selector.\n{:?}", err),
		};
	}

	fn eq_storage(&self, storage: &EndpointSerialized) -> bool {
		storage.service == "Pixiv" &&
			storage.endpoint_type == 0
	}
}

pub struct FollowAPIEndpoint {
	id: EndpointId,
	r18: bool,
	articles: Vec<Weak<RefCell<dyn ArticleData>>>,
	agent: Dispatcher<PixivAgent>,
	page: u16,
}

impl FollowAPIEndpoint {
	pub fn new(id: EndpointId, r18: bool, current_page: u16) -> Self {
		Self {
			id,
			r18,
			articles: Vec::new(),
			agent: PixivAgent::dispatcher(),
			page: current_page,
		}
	}
}

impl Endpoint for FollowAPIEndpoint {
	fn name(&self) -> String {
		"Follow API Endpoint".to_owned()
	}

	fn id(&self) -> &EndpointId {
		&self.id
	}

	fn articles(&mut self) -> &mut Vec<Weak<RefCell<dyn ArticleData>>> {
		&mut self.articles
	}

	fn refresh(&mut self, refresh_time: RefreshTime) {
		let id = self.id().clone();
		let query = web_sys::UrlSearchParams::new().unwrap();
		if self.page > 0 {
			query.append("p", &(self.page + 1).to_string());
		}
		if self.r18 {
			query.append("mode", "r18");
		}
		self.agent.send(Request::FetchPosts(
			refresh_time,
			id,
			format!("https://www.pixiv.net/ajax/follow_latest/illust?{}", query.to_string()),
		))
	}

	fn load_bottom(&mut self, refresh_time: RefreshTime) {
		self.page += 1;
		self.refresh(refresh_time)
	}

	fn eq_storage(&self, storage: &EndpointSerialized) -> bool {
		storage.service == "Pixiv" &&
			storage.endpoint_type == 1
	}
}

async fn fetch_posts(url: &str, storage: &SessionStorageService) -> FetchResult<Vec<Rc<RefCell<PixivArticleData>>>> {
	let response = reqwest::Client::builder().build()?
		.get(url)
		.send().await?;

	let json_str = response.text().await?.to_string();

	let response: serde_json::Value = serde_json::from_str(&json_str)?;
	let parsed: APIPayload<FollowAPIResponse> = serde_json::from_value(response.clone())?;
	Ok((parsed.body.thumbnails.illust
			.iter().zip(response["body"]["thumbnails"]["illust"].as_array().unwrap())
			.map(|(a, raw_json)| PixivArticleData::from((raw_json.clone(), a, storage)))
			.map(|p| Rc::new(RefCell::new(p)))
			.collect(),
		None))
}

async fn fetch_post(url: &str, storage: &SessionStorageService) -> FetchResult<Rc<RefCell<PixivArticleData>>> {
	let response = reqwest::Client::builder().build()?
		.get(url)
		.send().await?;

	let json_str = response.text().await?.to_string();

	let response: serde_json::Value = serde_json::from_str(&json_str)?;
	let parsed: APIPayload<FullPostAPI> = serde_json::from_value(response.clone())?;
	Ok((Rc::new(RefCell::new(PixivArticleData::from((response["body"].clone(), parsed.body, storage)))), None))
}