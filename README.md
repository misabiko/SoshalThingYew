# SoshalThingYew [![CI@main](https://github.com/misabiko/SoshalThingYew/actions/workflows/ci.yml/badge.svg?branch=main "CI@main")](https://github.com/misabiko/SoshalThingYew/actions/workflows/ci.yml) <a href="https://bulma.io"> <img src="https://bulma.io/images/made-with-bulma.png" alt="Made with Bulma" width="128" height="24"> </a>

Tweetdeck-style timeline app to display feeds from various services in columns.

![screenshot](/docs/screenshot.png?raw=true)
---

### Dev
Serve the app  on `localhost:8080` with `cargo run -p utils --bin server`  
Needs a `credentials.json` in the working directory with `consumer_key` and `consumer_secret` for a Twitter app.  
That or setting `consumer_key` and `consumer_secret` as environment variables.  

If not using any endpoints with proxy `trunk serve` should work too.

### Release

To deploy, only `server.exe`, `dist/` folder and `credentials.json` or environment variables are needed.