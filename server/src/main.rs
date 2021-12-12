use actix_web::{get, web, App, HttpResponse, HttpServer, Result, middleware::Logger, web::Path, Responder};
use egg_mode::list::{self, ListID};
use serde::Deserialize;

#[derive(serde::Deserialize)]
struct Credentials {
	consumer_key: String,
	consumer_secret: String,
}

#[get("/art")]
async fn art(token: web::Data<egg_mode::Token>) -> Result<HttpResponse> {
    let timeline = list::statuses(ListID::from_slug("misabiko", "art"), false, &token);
	let (_timeline, feed) = timeline.start().await.unwrap();

	Ok(HttpResponse::Ok()
		.append_header(("x-rate-limit-limit".to_owned(), feed.rate_limit_status.limit.clone()))
		.append_header(("x-rate-limit-remaining".to_owned(), feed.rate_limit_status.remaining.clone()))
		.append_header(("x-rate-limit-reset".to_owned(), feed.rate_limit_status.reset.clone()))
		.json(&feed.response)
	)
}

#[get("/twitter/status/{id}")]
async fn status(id: Path<u64>, token: web::Data<egg_mode::Token>) -> HttpResponse {
	match egg_mode::tweet::show(id.into_inner(), &token).await {
		egg_mode::error::Result::Ok(r) => HttpResponse::Ok()
			.append_header(("x-rate-limit-limit".to_owned(), r.rate_limit_status.limit.clone()))
			.append_header(("x-rate-limit-remaining".to_owned(), r.rate_limit_status.remaining.clone()))
			.append_header(("x-rate-limit-reset".to_owned(), r.rate_limit_status.reset.clone()))
			.json(&r.response),
		egg_mode::error::Result::Err(err) => HttpResponse::InternalServerError().body(err.to_string())
	}
}

#[derive(Deserialize)]
struct UserTimelineQuery {
	replies: Option<bool>,
	rts: Option<bool>,
	count: Option<i32>,
	min_id: Option<u64>,
	max_id: Option<u64>,
}

#[get("/twitter/user/{username}")]
async fn user_timeline(username: Path<String>, query: web::Query<UserTimelineQuery>, token: web::Data<egg_mode::Token>) -> Result<HttpResponse> {
	let timeline = egg_mode::tweet::user_timeline(
		egg_mode::user::UserID::ScreenName(username.into_inner().into()),
		query.replies.unwrap_or(true),
		query.rts.unwrap_or(true),
		&token
	)
	.with_page_size(query.count.unwrap_or(200));

	/*let (_timeline, feed) = if query.max_id.is_some() {
		timeline.older(query.max_id).await.unwrap()
	}else if query.min_id.is_some() {
		timeline.newer(query.min_id).await.unwrap()
	}else {
		timeline.start().await.unwrap()
	};*/
	let feed = timeline.call(query.min_id, query.max_id).await.unwrap();

	Ok(HttpResponse::Ok()
		.append_header(("x-rate-limit-limit".to_owned(), feed.rate_limit_status.limit.clone()))
		.append_header(("x-rate-limit-remaining".to_owned(), feed.rate_limit_status.remaining.clone()))
		.append_header(("x-rate-limit-reset".to_owned(), feed.rate_limit_status.reset.clone()))
		.json(&feed.response))
}

#[get("/{filename}")]
async fn favviewer(path: web::Path<String>) -> impl Responder {
	let path = path.into_inner();
	match path.as_str() {
		"init" => {
			let js_file = std::fs::read_dir("./dist")
				.unwrap()
				.find_map(|f| {
					f.ok()
						.map(|entry| entry.path())
						.and_then(|path| {
							match path
								.clone()
								.extension()
								.and_then(|ext| ext.to_str())
								.map(|ext| ext == "js") {
								Some(true) => Some(path.clone()),
								_ => None
							}
						})
				});

			match js_file.as_ref().and_then(|f| f.to_str()) {
				Some(file) => actix_files::NamedFile::open(&file),
				None => Err(std::io::Error::from(std::io::ErrorKind::NotFound))
			}
		},
		_ => actix_files::NamedFile::open(format!("dist/{}", &path)),
	}
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let credentials = std::fs::read_to_string("credentials.json").expect("Couldn't find credentials.json");
	let credentials: Credentials = serde_json::from_str(&credentials)?;

	let con_token = egg_mode::KeyPair::new(credentials.consumer_key, credentials.consumer_secret);
	let token = web::Data::new(egg_mode::auth::bearer_token(&con_token).await.unwrap());

	std::env::set_var("RUST_LOG", "actix_web=info");
	env_logger::init();

	HttpServer::new(move || {
		//let cors = actix_cors::Cors::default()
		//	.allowed_origin("https://www.pixiv.net")
		//	.allowed_methods(vec!["GET"]);

		App::new()
			.wrap(Logger::default())
			.app_data(token.clone())
			//.service(
			//	web::scope("/favviewer")
			//		.wrap(cors)
			//		.service(favviewer)
			//)
			.service(
				web::scope("/proxy")
					.service(art)
					.service(status)
					.service(user_timeline)
			)
	})
	.bind("127.0.0.1:3000")?
	.run()
	.await
}