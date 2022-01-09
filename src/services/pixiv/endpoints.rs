use std::rc::{Rc, Weak};
use std::cell::RefCell;
use yew_agent::{Dispatched, Dispatcher};
use js_sys::Date;
use gloo_timers::callback::Timeout;
use serde::{Serialize, Deserialize};
use wasm_bindgen::JsValue;

use super::{PixivAgent, Request};
use super::article::{PixivArticleData, PixivArticleCached};
use crate::articles::{ArticleData, ArticleMedia, MediaQueueInfo, MediaType, ValidRatio};
use crate::services::{Endpoint, EndpointSerialized};
use crate::services::endpoint_agent::{EndpointId, RefreshTime};
use crate::services::storages::{ServiceStorage, get_service_storage};
use crate::log_error;

#[derive(Deserialize)]
pub struct APIPayload<T> {
	pub error: bool,
	pub message: String,
	pub body: T,
}

#[derive(Serialize, Deserialize)]
pub struct FullPostAPI {
	pub id: String,
	pub title: String,
	pub urls: FullPostAPIURLs,
	#[serde(rename = "userAccount")]
	pub user_account: String,
	#[serde(rename = "userName")]
	pub user_name: String,
	#[serde(rename = "userId")]
	pub user_id: String,
	#[serde(rename = "likeCount")]
	pub like_count: u32,
	#[serde(rename = "likeData")]
	pub like_data: bool,
	#[serde(rename = "bookmarkCount")]
	pub bookmark_count: u32,
	#[serde(rename = "bookmarkData")]
	pub bookmark_data: bool,
	#[serde(rename = "createDate")]
	pub create_date: String,
}

#[derive(Serialize, Deserialize)]
pub struct FullPostAPIURLs {
	pub mini: String,
	pub thumb: String,
	pub small: String,
	pub regular: String,
	pub original: String,
}

#[derive(Deserialize)]
pub struct FollowAPIResponse {
	//pub page: FollowAPIPage,
	pub thumbnails: FollowAPIThumbnails,
}

/*#[derive(Deserialize)]
struct FollowAPIPage {
	pub ids: Vec<u32>,
}*/

#[derive(Deserialize)]
pub struct FollowAPIThumbnails {
	pub illust: Vec<FollowAPIIllust>,
}

#[derive(Serialize, Deserialize)]
pub struct FollowAPIIllust {
	pub id: String,
	pub title: String,
	pub url: String,
	#[serde(rename = "userId")]
	pub user_id: String,
	#[serde(rename = "userName")]
	pub user_name: String,
	#[serde(rename = "profileImageUrl")]
	pub profile_image_url: String,
	#[serde(rename = "bookmarkData")]
	pub bookmark_data: bool,
	#[serde(rename = "createDate")]
	pub create_date: String,
}

impl From<(serde_json::Value, &FullPostAPI, &ServiceStorage)> for PixivArticleData {
	fn from((raw_json, data, storage): (serde_json::Value, &FullPostAPI, &ServiceStorage)) -> Self {
		let cached: Option<PixivArticleCached> = storage.session.cached_articles.get(&data.id)
			.and_then(|json| serde_json::from_value(json.clone()).ok());
		let author_avatar_url = match cached {
			Some(PixivArticleCached { author_avatar_url, .. }) => author_avatar_url,
			None => "".to_owned(),
		};

		PixivArticleData {
			id: data.id.parse::<u32>().unwrap(),
			creation_time: Date::new(&JsValue::from_str(&data.create_date)),
			title: data.title.clone(),
			media: ArticleMedia {
				media_type: MediaType::Image,
				src: data.urls.original.clone(),
				ratio: ValidRatio::one(), //TODO Pixiv image ratio
				queue_load_info: MediaQueueInfo::Thumbnail,
			},
			author_name: data.user_name.clone(),
			author_id: data.user_id.parse::<u32>().unwrap(),
			author_avatar_url,
			marked_as_read: storage.session.articles_marked_as_read.contains(data.id.as_str()),
			hidden: storage.local.hidden_articles.contains(data.id.as_str()),
			is_fully_fetched: true,
			raw_json,
			like_count: data.like_count,
			liked: data.like_data,
			bookmark_count: data.bookmark_count,
			bookmarked: data.bookmark_data,
		}
	}
}

impl From<(serde_json::Value, FullPostAPI, &ServiceStorage)> for PixivArticleData {
	fn from((raw_json, data, storage): (serde_json::Value, FullPostAPI, &ServiceStorage)) -> Self {
		PixivArticleData::from((raw_json, &data, storage))
	}
}

impl From<(serde_json::Value, &FollowAPIIllust, &ServiceStorage)> for PixivArticleData {
	fn from((raw_json, data, storage): (serde_json::Value, &FollowAPIIllust, &ServiceStorage)) -> Self {
		let cached: Option<PixivArticleCached> = storage.session.cached_articles.get(&data.id)
			.and_then(|json| serde_json::from_value(json.clone()).ok());
		let (media, is_fully_fetched) = match cached {
			Some(PixivArticleCached { media, .. }) => (media, true),
			None => (ArticleMedia {
				media_type: MediaType::Image,
				src: data.url.clone(),
				ratio: ValidRatio::one(),
				queue_load_info: MediaQueueInfo::Thumbnail,
			}, false)
		};

		PixivArticleData {
			id: data.id.parse::<u32>().unwrap(),
			creation_time: Date::new(&JsValue::from_str(&data.create_date)),
			title: data.title.clone(),
			media,
			author_name: data.user_name.clone(),
			author_id: data.user_id.parse::<u32>().unwrap(),
			author_avatar_url: data.profile_image_url.clone(),
			marked_as_read: storage.session.articles_marked_as_read.contains(data.id.as_str()),
			hidden: storage.local.hidden_articles.contains(data.id.as_str()),
			is_fully_fetched,
			raw_json,
			like_count: 0,
			liked: false,
			bookmark_count: 0,
			bookmarked: data.bookmark_data.clone(),
		}
	}
}

fn parse_article(element: web_sys::Element, storage: &ServiceStorage) -> Option<Rc<RefCell<PixivArticleData>>> {
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

	let cached: Option<serde_json::Result<PixivArticleCached>> = storage.session.cached_articles.get(&id_str)
		.map(|json| serde_json::from_value(json.clone()));
	let (media, is_fully_fetched) = match cached {
		Some(Ok(PixivArticleCached { media, .. })) => (media, true),
		Some(Err(_err)) => {
			let media = match imgs.get_with_index(0) {
				Some(img) => match img.get_attribute("src") {
					Some(src) => ArticleMedia {
						media_type: MediaType::Image,
						src,
						ratio: ValidRatio::one(),
						queue_load_info: MediaQueueInfo::Thumbnail,
					},
					None => return None,
				}
				None => return None,
			};
			(media, false)
		}
		None => {
			let media = match imgs.get_with_index(0) {
				Some(img) => match img.get_attribute("src") {
					Some(src) => ArticleMedia {
						media_type: MediaType::Image,
						src,
						ratio: ValidRatio::one(),
						queue_load_info: MediaQueueInfo::Thumbnail,
					},
					None => return None,
				}
				None => return None,
			};
			(media, false)
		}
	};

	Some(Rc::new(RefCell::new(PixivArticleData {
		id,
		creation_time: js_sys::Date::new_0(),
		media,
		author_avatar_url,
		title,
		author_id,
		author_name,
		marked_as_read: storage.session.articles_marked_as_read.contains(&id.to_string()),
		hidden: storage.local.hidden_articles.contains(&id.to_string()),
		is_fully_fetched,
		raw_json: serde_json::Value::Null,
		like_count: 0,
		liked: false,
		bookmark_count: 0,
		bookmarked: false,
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
					let id = self.id;
					let timeout = Some(Timeout::new(1_000, move || agent.send(Request::RefreshEndpoint(id, refresh_time))));
					self.timeout = timeout;
				}
			}
			Ok(Some(posts)) => {
				let children = posts.children();
				log::debug!("Found {} posts.", children.length());
				let storage = get_service_storage("Pixiv");
				for i in 0..children.length() {
					if let Some(article) = children.get_with_index(i).and_then(|a| parse_article(a, &storage)) {
						articles.push(article);
					}
				}

				self.agent.send(Request::AddArticles(refresh_time, self.id, articles));
				self.timeout = None;
			}
			Err(err) => log_error!("Failed to use query_selector", err),
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
		let query = web_sys::UrlSearchParams::new().unwrap();
		if self.page > 0 {
			query.append("p", &(self.page + 1).to_string());
		}
		if self.r18 {
			query.append("mode", "r18");
		}
		self.agent.send(Request::FetchPosts(
			refresh_time,
			self.id,
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