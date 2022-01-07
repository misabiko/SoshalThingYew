use std::fmt::{Debug, Display, Formatter};
use std::num::ParseIntError;
use gloo_storage::errors::StorageError;
use reqwest::header::ToStrError;
use wasm_bindgen::JsValue;

use crate::services::RateLimit;

pub type Result<T> = std::result::Result<T, Error>;
pub type RatelimitedResult<T> = std::result::Result<(T, Option<RateLimit>), Error>;

#[macro_export]
macro_rules! log_error {
	($message:expr, $error:expr) => {{
		log::error!("{}", $crate::error::Error::Generic {
			message: Some($message.to_owned()),
			error: $error.into()
		});
	}};
}

#[macro_export]
macro_rules! log_warn {
	($message:expr, $error:expr) => {{
		log::warn!("{}", $crate::error::Error::Generic {
			message: Some($message.to_owned()),
			error: $error.into()
		});
	}};
}

#[derive(Debug)]
pub enum Error {
	Generic {
		message: Option<String>,
		error: ActualError,
	},
	UnauthorizedFetch {
		message: Option<String>,
		error: ActualError,
		article_ids: Vec<String>,
	},
	ArticleFetch {
		message: Option<String>,
		error: ActualError,
		article_ids: Vec<String>,
	},
	RatelimitedArticleFetch {
		message: Option<String>,
		error: ActualError,
		article_ids: Vec<String>,
		ratelimit: RateLimit,
	},
}

impl Error {
	pub fn with_message(mut self, new_message: &str) -> Self {
		match &mut self {
			Error::Generic { message, .. } |
			Error::UnauthorizedFetch { message, .. } |
			Error::ArticleFetch { message, .. } |
			Error::RatelimitedArticleFetch { message, .. }
			=> *message = Some(new_message.to_owned()),
		};
		self
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let article_ids = match self {
			Error::Generic { .. } => "".to_owned(),
			Error::UnauthorizedFetch { article_ids, .. } |
			Error::ArticleFetch { article_ids, .. } |
			Error::RatelimitedArticleFetch { article_ids, .. } => if article_ids.is_empty() {
				"".to_owned()
			} else {
				format!("While fetching articles {:?}.", article_ids)
			},
		};
		match self {
			Error::Generic { message, error } => f.write_fmt(format_args!("{}.\n{}", message.as_ref().unwrap_or(&"Generic error".to_owned()), error)),
			Error::UnauthorizedFetch { message, error, .. } => f.write_fmt(format_args!("{}. {}\n{}", message.as_ref().unwrap_or(&"Unauthorized fetch error".to_owned()), article_ids, error)),
			Error::ArticleFetch { message, error, .. } => f.write_fmt(format_args!("{}. {}\n{}", message.as_ref().unwrap_or(&"Generic error".to_owned()), article_ids, error)),
			Error::RatelimitedArticleFetch { message, error, ratelimit, .. } => f.write_fmt(format_args!("{}. {}\nWith rate limit: {:?}\n{}", message.as_ref().unwrap_or(&"Generic error".to_owned()), article_ids, ratelimit, error)),
		}
	}
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub enum ActualError {
	Text(String),
	Reqwest(reqwest::Error),
	SerdeJson(serde_json::Error),
	ToStr(ToStrError),
	ParseInt(ParseIntError),
	JsValue(JsValue),
	Storage(StorageError),
}

impl Display for ActualError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			ActualError::Text(str) => f.write_str(str),
			ActualError::Reqwest(err) => Display::fmt(err, f),
			ActualError::SerdeJson(err) => f.write_fmt(format_args!("{}:{} - {}", err.line(), err.column(), err)),
			ActualError::ToStr(err) => Display::fmt(err, f),
			ActualError::ParseInt(err) => Display::fmt(err, f),
			ActualError::JsValue(value) => Debug::fmt(value, f),
			ActualError::Storage(err) => Display::fmt(err, f),
		}
	}
}

impl<T> From<T> for Error
	where T: Into<ActualError> {
	fn from(error: T) -> Self {
		Error::Generic {
			message: None,
			error: error.into(),
		}
	}
}

impl From<String> for ActualError {
	fn from(error: String) -> Self {
		ActualError::Text(error)
	}
}

impl From<&'static str> for ActualError {
	fn from(error: &'static str) -> Self {
		Self::from(error.to_owned())
	}
}

impl From<reqwest::Error> for ActualError {
	fn from(error: reqwest::Error) -> Self {
		ActualError::Reqwest(error)
	}
}

impl From<serde_json::Error> for ActualError {
	fn from(error: serde_json::Error) -> Self {
		ActualError::SerdeJson(error)
	}
}

impl From<ToStrError> for ActualError {
	fn from(error: ToStrError) -> Self {
		ActualError::ToStr(error)
	}
}

impl From<ParseIntError> for ActualError {
	fn from(error: ParseIntError) -> Self {
		ActualError::ParseInt(error)
	}
}

impl From<JsValue> for ActualError {
	fn from(error: JsValue) -> Self {
		ActualError::JsValue(error)
	}
}

impl From<StorageError> for ActualError {
	fn from(error: StorageError) -> Self {
		ActualError::Storage(error)
	}
}