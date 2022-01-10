use std::cell::RefCell;
use std::rc::{Rc, Weak};
use yew_agent::{Dispatcher, Dispatched};

use super::{YoutubeAgent, Request};
use crate::{Endpoint, EndpointId};
use crate::articles::ArticleData;
use crate::services::{EndpointSerialized, RefreshTime};
use crate::services::youtube::article::{YoutubeArticleData, YoutubeChannel};

pub struct HardCodedEndpoint {
	id: EndpointId,
	articles: Vec<Weak<RefCell<dyn ArticleData>>>,
	agent: Dispatcher<YoutubeAgent>,
}

impl HardCodedEndpoint {
	pub fn new(id: EndpointId) -> Self {
		Self {
			id,
			articles: Vec::new(),
			agent: YoutubeAgent::dispatcher(),
		}
	}
}

impl Endpoint for HardCodedEndpoint {
	fn name(&self) -> String {
		"Hardcoded endpoint".to_owned()
	}

	fn id(&self) -> &EndpointId {
		&self.id
	}

	fn articles(&mut self) -> &mut Vec<Weak<RefCell<dyn ArticleData>>> {
		&mut self.articles
	}

	fn refresh(&mut self, refresh_time: RefreshTime) {
		self.agent.send(Request::AddArticles(refresh_time, self.id, vec![Rc::new(RefCell::new(YoutubeArticleData {
			id: "-iPh3Vuhp80".to_owned(),
			creation_time: js_sys::Date::new_0(),
			description: "柊キライと申します。".to_string(),
			channel: YoutubeChannel {
				id: "UCgf4ASaUXSl960o8wXWHGdQ".to_owned(),
				name: "柊キライ".to_owned(),
				avatar_url: "https://yt3.ggpht.com/ytc/AKedOLSmqlzbT5JYWmHe6t_gZIzIjWiPwWSjTJXsiTDo=s176-c-k-c0x00ffffff-no-rj".to_owned(),
			},
			//TODO Abstract get_service_storage to ArticleData?
			marked_as_read: false,
			hidden: false,
		}))]));
	}

	fn eq_storage(&self, storage: &EndpointSerialized) -> bool {
		storage.service == "Youtube" &&
			storage.endpoint_type == 0
	}
}