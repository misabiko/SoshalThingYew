use actix_web::{get, web, App, HttpResponse, HttpServer, middleware::Logger};
use actix_identity::{Identity, CookieIdentityPolicy, IdentityService};
use serde::Serialize;
use std::fmt::{Display, Formatter};
use std::fs::File;
use actix_web::web::Data;
use log::LevelFilter;
use rand::Rng;
use serde::Deserialize;
use simplelog::{ColorChoice, CombinedLogger, Config, TerminalMode, TermLogger, WriteLogger};

mod twitter;
mod youtube;
use crate::twitter::{TwitterCredentials, TwitterData};
use crate::youtube::{YouTubeCredentials, YouTubeData};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	Text(String),
	Actix(actix_web::Error),
	EggMode(egg_mode::error::Error),
	IO(std::io::Error),
	Serde(serde_json::Error),
}

impl std::error::Error for Error {}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::Text(str) => f.write_str(&str),
			Error::Actix(err) => err.fmt(f),
			Error::EggMode(err) => err.fmt(f),
			Error::IO(err) => err.fmt(f),
			Error::Serde(err) => err.fmt(f),
		}
	}
}

impl From<String> for Error {
	fn from(err: String) -> Self {
		Error::Text(err)
	}
}

impl From<actix_web::Error> for Error {
	fn from(err: actix_web::Error) -> Self {
		Error::Actix(err)
	}
}

impl From<egg_mode::error::Error> for Error {
	fn from(err: egg_mode::error::Error) -> Self {
		Error::EggMode(err)
	}
}

impl From<std::io::Error> for Error {
	fn from(err: std::io::Error) -> Self {
		Error::IO(err)
	}
}

impl From<serde_json::Error> for Error {
	fn from(err: serde_json::Error) -> Self {
		Error::Serde(err)
	}
}

impl actix_web::ResponseError for Error {}

#[derive(Deserialize)]
pub struct Credentials {
	twitter: Option<TwitterCredentials>,
	youtube: Option<YouTubeCredentials>,
}

pub struct State {
	pub twitter: Option<TwitterData>,
	pub youtube: Option<YouTubeData>,
}

#[derive(Serialize)]
struct AuthInfo {
	pub twitter: Option<String>,
	pub youtube: bool,
}

#[get("/auth_info")]
async fn auth_info(id: Identity, data: Data<State>) -> HttpResponse {
	HttpResponse::Ok().json(AuthInfo {
		twitter: id.identity(),
		//TODO Use identity for youtube
		youtube: data.youtube.as_ref().map(|s| s.is_logged_in()).unwrap_or(false)
	})
}

/*async fn index(_req: HttpRequest) -> Result<NamedFile> {
	Ok(NamedFile::open("index.html")?)
}*/

#[actix_web::main]
async fn main() -> Result<()> {
	//For testing actix freeze, leaving this in for a few commits
	/*CombinedLogger::init(
		vec![
			TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
			WriteLogger::new(LevelFilter::Trace, Config::default(), File::create(format!("soshalthing.log")).unwrap()),
		]
	).unwrap();*/
	TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto).unwrap();

	//TODO std::mem::take instead of cloning
	let credentials: Option<Credentials> = std::fs::read_to_string("credentials.json").ok()
		.and_then(|c| serde_json::from_str(&c).ok());
	let data = web::Data::new(State {
		twitter: twitter::state(credentials.as_ref().and_then(|c| c.twitter.clone())).await.ok(),
		youtube: youtube::state(credentials.as_ref().and_then(|c| c.youtube.clone())).await.ok(),
	});

	let cookie_key = rand::thread_rng().gen::<[u8; 32]>();

	HttpServer::new(move || {
		App::new()
			.wrap(IdentityService::new(CookieIdentityPolicy::new(&cookie_key).secure(true)))
			.wrap(Logger::new(r#"%a %t "%r" %s "%{Referer}i" %T ms"#))
			.app_data(data.clone())
			.service(
				web::scope("/proxy")
					.service(twitter::service())
					.service(youtube::service())
					.service(auth_info)
			)
			//TODO Fix /twitter/status/{id} shortcut
			//.route("/twitter/{_:.*}", web::to(index))
			.service(actix_files::Files::new("/", "./dist").index_file("index.html"))
	})
	.bind("127.0.0.1:8080")?
	.run()
	.await
	.map_err(|err| Error::from(err))?;

	//TODO Actually log this
	log::info!("Server listening at http://localhost:8080");

	Ok(())
}