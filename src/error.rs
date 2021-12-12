#[derive(Debug)]
pub enum Error {
	Reqwest(reqwest::Error),
	SerdeJson(serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<reqwest::Error> for Error {
	fn from(err: reqwest::Error) -> Self {
		Error::Reqwest(err)
	}
}

impl From<serde_json::Error> for Error {
	fn from(err: serde_json::Error) -> Self {
		Error::SerdeJson(err)
	}
}

/*impl FromIterator<T> for Result<T> {
	fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
		Ok(Vec::from(iter))
	}
}
impl From<serde_json::Result<T>> for Result<T> {
	fn from(result: serde_json::Result<T>) -> Self {
		match result {
			Ok(value) => Ok(value),
			Err(err) => Err(err),
		}
	}
}*/