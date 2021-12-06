use std::rc::Rc;
use yew::worker::*;
use wasm_bindgen::prelude::*;

use crate::articles::SocialArticleData;

pub trait Endpoint : Agent {
	fn name(&self) -> String;
}

pub enum EndpointRequest {
	Refresh,
}

pub enum EndpointResponse {
	NewArticles(Vec<Rc<dyn SocialArticleData>>),
}

pub enum EndpointMsg {
	Refreshed(Vec<Rc<dyn SocialArticleData>>),
	RefreshFail(JsValue),
}