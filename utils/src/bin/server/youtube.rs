use std::future::ready;
use std::sync::Mutex;
use actix_web::{HttpResponse, web, get};
use actix_web::dev::{HttpServiceFactory, Service};
use actix_web::http::header;
use actix_web::web::{Data, Path, Query};
use serde::Deserialize;
use youtube_api::models::ListPlaylistItemsRequestBuilder;
use youtube_api::YoutubeApi;

use crate::{State, Result, Error};

#[derive(Deserialize, Clone)]
pub struct YouTubeCredentials {
	api_key: String,
	client_id: String,
	client_secret: String,
}

//Get rid of the wrapping tuple?
pub struct YouTubeData(Mutex<YouTubeState>);

enum YouTubeState {
	NotLoggedIn(YoutubeApi),
	LoggingIn(YoutubeApi, String),
	LoggedIn(YoutubeApi),
}

impl YouTubeData {
	pub fn is_logged_in(&self) -> bool {
		match &*self.0.lock().expect("locking youtube state") {
			YouTubeState::NotLoggedIn(api) |
			YouTubeState::LoggingIn(api, _) |
			YouTubeState::LoggedIn(api)
				=> api.has_token(),
		}
	}
}

pub async fn state(credentials: Option<YouTubeCredentials>) -> Result<YouTubeData> {
	let credentials = credentials.ok_or(Error::from("No YouTube credentials.".to_owned()))?;

	Ok(YouTubeData(Mutex::new(YouTubeState::NotLoggedIn(
		YoutubeApi::new_with_oauth(
			credentials.api_key.clone(),
			credentials.client_id.clone(),
			credentials.client_secret.clone(),
			Some("http://localhost:8080/proxy/youtube/callback"),
		).map_err(|err| Error::from(err.to_string()))?))))
}

pub fn service() -> impl HttpServiceFactory {
	web::scope("/youtube")
		.wrap_fn(|req, service| {
			let has_data = req.app_data::<Data<State>>().map(|s| s.youtube.is_some()).unwrap_or(false);
			let fut = service.call(req);

			if has_data {
				fut
			} else {
				//TODO Properly send error response
				Box::pin(ready(Err(actix_web::Error::from(Error::from("No YouTube data".to_owned())))))
			}
		})
		.service(playlist)
		.service(login_callback)
		.service(login)
}

#[get("playlist/{id}")]
async fn playlist(playlist_id: Path<String>, data: Data<State>) -> Result<HttpResponse> {
	if let YouTubeState::LoggedIn(api) = &*data.youtube.as_ref().unwrap().0.lock().expect("locking youtube state") {
		let request = ListPlaylistItemsRequestBuilder {
			playlist_id: Some(playlist_id.clone()),
			max_results: Some(50),
		};
		let response = api.list_playlist_items(request).await
			.map_err(|err| Error::from(err.to_string()))?;
		Ok(HttpResponse::Ok().json(response.items))
	}else {
		Ok(HttpResponse::Unauthorized().finish())
	}
}

#[get("login")]
async fn login(data: Data<State>) -> Result<HttpResponse> {
	//TODO move api instead of cloning
	let (api, (authorize_url, verifier)) = match &*data.youtube.as_ref().unwrap().0.lock().expect("locking youtube state") {
		YouTubeState::NotLoggedIn(api) |
		YouTubeState::LoggingIn(api, _) |
		YouTubeState::LoggedIn(api) =>
			(api.clone(), api.get_oauth_url().map_err(|err| Error::from(err.to_string()))?)
	};

	*data.youtube.as_ref().unwrap().0.lock().expect("locking youtube state") = YouTubeState::LoggingIn(api, verifier);

	log::info!("Redirecting to {}", &authorize_url);
	Ok(HttpResponse::TemporaryRedirect()
		.append_header((header::LOCATION, authorize_url))
		.finish())
}

#[derive(Deserialize)]
struct LoginCallbackQuery {
	code: String,
	//scope: String,
}

#[get("callback")]
async fn login_callback(query: Query<LoginCallbackQuery>, data: Data<State>) -> Result<HttpResponse> {
	//TODO move api instead of cloning
	let result: Result<Option<(YoutubeApi, String)>> = match &*data.youtube.as_ref().unwrap().0.lock().expect("locking youtube state") {
		YouTubeState::NotLoggedIn(_) => Err(Error::from("No verifier".to_owned()))?,
		YouTubeState::LoggedIn(_) => Ok(None),
		YouTubeState::LoggingIn(api, verifier) => Ok(Some((api.clone(), verifier.clone()))),
	};

	if let Some((mut api, verifier)) = result? {
		api.request_token(query.code.clone(), verifier).await.map_err(|err| Error::from(err.to_string()))?;
		*data.youtube.as_ref().unwrap().0.lock().expect("locking youtube state") = YouTubeState::LoggedIn(api);
	}

	Ok(HttpResponse::TemporaryRedirect()
		.append_header((header::LOCATION, "http://localhost:8080/"))
		.finish())
}