use std::cell::RefCell;
use std::rc::{Rc, Weak};
use reqwest::{StatusCode, Url};
use yew_agent::{Dispatcher, Dispatched};

use super::{YouTubeAgent, Request, SERVICE_INFO};
use crate::{base_url, Endpoint, EndpointId};
use crate::articles::ArticleData;
use crate::error::{Result, Error};
use crate::services::{EndpointSerialized, RefreshTime};
use crate::services::storages::ServiceStorage;
use crate::services::youtube::article::{PlaylistItem, YouTubeArticleData};

pub async fn fetch_videos(url: Url, storage: &ServiceStorage) -> Result<Vec<Rc<RefCell<YouTubeArticleData>>>> {
	let response = reqwest::Client::builder()
		//.timeout(Duration::from_secs(10))
		.build()?
		.get(url)
		.send().await?
		.error_for_status()
		.map_err(|err| if let Some(StatusCode::UNAUTHORIZED) = err.status() {
			Error::UnauthorizedFetch {
				message: None,
				error: err.into(),
				article_ids: vec![],
			}
		}else {
			err.into()
		})?;

	let json_str = response.text().await?.to_string();

	serde_json::from_str(&json_str)
		.map(|value: serde_json::Value|
			value.as_array().unwrap().iter().map(|json|
				Rc::new(RefCell::new(YouTubeArticleData::from((
					serde_json::from_value::<PlaylistItem>(json.clone()).unwrap(),
					json.clone(),
					storage
				))))).collect(),
		)
		.map_err(|err| Error::from(err))
}

pub struct PlaylistEndpoint {
	id: EndpointId,
	articles: Vec<Weak<RefCell<dyn ArticleData>>>,
	agent: Dispatcher<YouTubeAgent>,
	playlist_id: String,
}

impl PlaylistEndpoint {
	pub fn new(id: EndpointId, playlist_id: String) -> Self {
		Self {
			id,
			articles: Vec::new(),
			agent: YouTubeAgent::dispatcher(),
			playlist_id,
		}
	}

	pub fn from_json(id: EndpointId, params: serde_json::Value) -> Self {
		Self::new(id, params["id"].as_str().unwrap().to_owned())
	}
}

impl Endpoint for PlaylistEndpoint {
	//TODO Store string in endpoint
	fn name(&self) -> String {
		format!("Playlist {}", &self.playlist_id)
	}

	fn id(&self) -> &EndpointId {
		&self.id
	}

	fn articles(&mut self) -> &mut Vec<Weak<RefCell<dyn ArticleData>>> {
		&mut self.articles
	}

	fn refresh(&mut self, refresh_time: RefreshTime) {
		self.agent.send(Request::FetchArticles(
			refresh_time,
			self.id,
			Url::parse(&format!("{}/proxy/youtube/playlist/{}", base_url(), self.playlist_id)).unwrap()
		))
	}

	fn eq_storage(&self, storage: &EndpointSerialized) -> bool {
		storage.service == SERVICE_INFO.name &&
			storage.endpoint_type == 0
	}
}