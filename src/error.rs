use std::num::ParseIntError;
use reqwest::header::ToStrError;
use wasm_bindgen::JsValue;

use crate::services::RateLimit;

#[derive(Debug)]
pub enum Error {
	Generic(ErrorKind),
	ArticleFetch {
		err: ErrorKind,
		article_ids: Vec<String>,
	},
	RatelimitedArticleFetch {
		err: ErrorKind,
		article_ids: Vec<String>,
		ratelimit: RateLimit,
	},
}

impl Error {
	pub fn kind(&self) -> &ErrorKind {
		match self {
			Error::Generic(err) | Error::ArticleFetch { err, .. } | Error::RatelimitedArticleFetch { err, .. }
				=> err
		}
	}
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
pub type RatelimitedResult<T> = std::result::Result<(T, Option<RateLimit>), Error>;

impl<T> From<T> for Error
	where T : Into<ErrorKind> {
	fn from(err: T) -> Self {
		Error::Generic(err.into())
	}
}

impl From<String> for ErrorKind {
	fn from(err: String) -> Self {
		ErrorKind::Text(err)
	}
}

impl From<&'static str> for ErrorKind {
	fn from(err: &'static str) -> Self {
		Self::from(err.to_owned())
	}
}

impl From<reqwest::Error> for ErrorKind {
	fn from(err: reqwest::Error) -> Self {
		ErrorKind::Reqwest(err)
	}
}

impl From<serde_json::Error> for ErrorKind {
	fn from(err: serde_json::Error) -> Self {
		ErrorKind::SerdeJson(err)
	}
}

impl From<ToStrError> for ErrorKind {
	fn from(err: ToStrError) -> Self {
		ErrorKind::ToStr(err)
	}
}

impl From<ParseIntError> for ErrorKind {
	fn from(err: ParseIntError) -> Self {
		ErrorKind::ParseInt(err)
	}
}

impl From<JsValue> for ErrorKind {
	fn from(err: JsValue) -> Self {
		ErrorKind::JsValue(err)
	}
}