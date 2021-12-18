use std::num::ParseIntError;
use reqwest::header::ToStrError;

use crate::services::endpoints::RateLimit;

#[derive(Debug)]
pub struct Error {
	err: ErrorKind,
	ratelimit: Option<RateLimit>,
}

#[derive(Debug)]
pub enum ErrorKind {
	Text(&'static str),
	Reqwest(reqwest::Error),
	SerdeJson(serde_json::Error),
	ToStr(ToStrError),
	ParseInt(ParseIntError),
}

pub type Result<T> = std::result::Result<T, Error>;
pub type FetchResult<T> = std::result::Result<(T, Option<RateLimit>), Error>;

impl From<&'static str> for Error {
	fn from(err: &'static str) -> Self {
		Self {
			err: ErrorKind::Text(err),
			ratelimit: None,
		}
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