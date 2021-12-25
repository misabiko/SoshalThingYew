use std::num::ParseIntError;
use reqwest::header::ToStrError;
use wasm_bindgen::JsValue;

use crate::services::RateLimit;

#[derive(Debug)]
pub struct Error {
	err: ErrorKind,
	ratelimit: Option<RateLimit>,
}

#[derive(Debug)]
pub enum ErrorKind {
	Text(String),
	Reqwest(reqwest::Error),
	SerdeJson(serde_json::Error),
	ToStr(ToStrError),
	ParseInt(ParseIntError),
	JsValue(JsValue),
}

pub type Result<T> = std::result::Result<T, Error>;
pub type FetchResult<T> = std::result::Result<(T, Option<RateLimit>), Error>;

impl From<String> for Error {
	fn from(err: String) -> Self {
		Self {
			err: ErrorKind::Text(err),
			ratelimit: None,
		}
	}
}

impl From<&'static str> for Error {
	fn from(err: &'static str) -> Self {
		Self::from(err.to_owned())
	}
}

impl From<reqwest::Error> for Error {
	fn from(err: reqwest::Error) -> Self {
		Self {
			err: ErrorKind::Reqwest(err),
			ratelimit: None,
		}
	}
}

impl From<serde_json::Error> for Error {
	fn from(err: serde_json::Error) -> Self {
		Self {
			err: ErrorKind::SerdeJson(err),
			ratelimit: None,
		}
	}
}

impl From<ToStrError> for Error {
	fn from(err: ToStrError) -> Self {
		Self {
			err: ErrorKind::ToStr(err),
			ratelimit: None,
		}
	}
}

impl From<ParseIntError> for Error {
	fn from(err: ParseIntError) -> Self {
		Self {
			err: ErrorKind::ParseInt(err),
			ratelimit: None,
		}
	}
}

impl From<JsValue> for Error {
	fn from(err: JsValue) -> Self {
		Self {
			err: ErrorKind::JsValue(err),
			ratelimit: None,
		}
	}
}