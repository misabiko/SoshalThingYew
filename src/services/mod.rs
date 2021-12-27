use std::rc::Weak;
use std::cell::RefCell;
use reqwest::header::HeaderMap;
use serde::{Serialize, Deserialize};

pub mod endpoint_agent;
pub mod article_actions;

pub mod twitter;
pub mod pixiv;

pub use endpoint_agent::{EndpointId, EndpointAgent, RefreshTime};
use crate::error::Error;
use crate::articles::ArticleData;
use crate::timeline::sort_methods::sort_by_id;
use crate::timeline::filters::FilterSerialized;

#[derive(Clone, Debug)]
pub struct RateLimit {
	pub limit: i32,
	pub remaining: i32,
	pub reset: i32,
}

impl Default for RateLimit {
	fn default() -> Self {
		Self {
			limit: 1,
			remaining: 1,
			reset: 0,
		}
	}
}

impl TryFrom<&HeaderMap> for RateLimit {
	type Error = Error;

	fn try_from(headers: &HeaderMap) -> std::result::Result<Self, Self::Error> {
		let ret = Self {
			limit: headers.get("x-rate-limit-limit")
				.ok_or(Error::from("Couldn't get ratelimit's limit"))?
				.to_str()?
				.parse()?,
			remaining: headers.get("x-rate-limit-remaining")
				.ok_or(Error::from("Couldn't get ratelimit's remaining"))?
				.to_str()?
				.parse()?,
			reset: headers.get("x-rate-limit-reset")
				.ok_or(Error::from("Couldn't get ratelimit's reset"))?
				.to_str()?
				.parse()?,
		};
		Ok(ret)
	}
}

impl RateLimit {
	pub fn can_refresh(&mut self) -> bool {
		if (self.reset as f64) < js_sys::Date::now() {
			self.remaining = self.limit;
			true
		}else {
			self.remaining > 0
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct EndpointSerialized {
	pub service: String,
	pub endpoint_type: usize,
	pub params: serde_json::Value,
	#[serde(default)]
	pub filters: Vec<FilterSerialized>,
}

pub trait Endpoint {
	fn name(&self) -> String;

	fn id(&self) -> &EndpointId;

	fn articles(&mut self) -> &mut Vec<Weak<RefCell<dyn ArticleData>>>;

	fn add_articles(&mut self, articles: Vec<Weak<RefCell<dyn ArticleData>>>)  {
		for a in articles {
			if !self.articles().iter().any(|existing| Weak::ptr_eq(&existing, &a)) {
				self.articles().push(a);
			}
		}
		self.articles().sort_by(sort_by_id)
	}

	fn ratelimit(&self) -> Option<&RateLimit> { None }

	fn get_mut_ratelimit(&mut self) -> Option<&mut RateLimit> { None }

	fn update_ratelimit(&mut self, _ratelimit: RateLimit) {}

	fn can_refresh(&self) -> bool { true }

	fn refresh(&mut self, refresh_time: RefreshTime);

	fn load_top(&mut self, refresh_time: RefreshTime) {
		log::debug!("{} doesn't implement load_top()", self.name());
		self.refresh(refresh_time)
	}

	fn load_bottom(&mut self, refresh_time: RefreshTime) {
		log::debug!("{} doesn't implement load_bottom()", self.name());
		self.refresh(refresh_time)
	}

	fn eq_storage(&self, storage: &EndpointSerialized) -> bool;

	fn default_interval(&self) -> u32 {
		90_000
	}
}
