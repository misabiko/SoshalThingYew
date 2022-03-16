const {expect, test} = require('@playwright/test');

test.describe('article actions', () => {
	test('like feedback', async ({page}) => {
		await page.goto('/');
		await page.mainFrame().evaluate(() => {
			window.localStorage.setItem('SoshalThingYew Timelines', JSON.stringify([{
				"title": "Timeline",
				"endpoints": [
					{
						"service": "Twitter",
						"endpoint_type": 4,
						"params": { "id": "1228393702244134912" },
						"on_start": true,
						"on_refresh": false
					}
				]
			}]));
		});

		await page.route('/proxy/twitter/status/', route => route.fulfill({
			body: {
				"coordinates": null,
				"created_at": "Fri Feb 14 19:00:55 +0000 2020",
				"current_user_retweet": null,
				"display_text_range": [
					0,
					97
				],
				"entities": {
					"hashtags": [],
					"symbols": [],
					"urls": [],
					"user_mentions": [],
					"media": null
				},
				"extended_entities": null,
				"favorite_count": 402,
				"favorited": false,
				"filter_level": null,
				"id": 1228393702244135000,
				"in_reply_to_user_id": null,
				"in_reply_to_screen_name": null,
				"in_reply_to_status_id": null,
				"lang": "en",
				"place": null,
				"possibly_sensitive": null,
				"quoted_status_id": null,
				"quoted_status": null,
				"retweet_count": 112,
				"retweeted": false,
				"retweeted_status": null,
				"source": {
					"name": "Twitter Web App",
					"url": "https://mobile.twitter.com"
				},
				"text": "What did the developer write in their Valentine’s card?\n  \nwhile(true) {\n    I = Love(You);  \n}",
				"truncated": false,
				"user": {
					"contributors_enabled": false,
					"created_at": "2013-12-14T04:35:55Z",
					"default_profile": false,
					"default_profile_image": false,
					"description": "The voice of the #TwitterDev team and your official source for updates, news, and events, related to the #TwitterAPI.",
					"entities": {
						"description": {
							"urls": []
						},
						"url": {
							"urls": [
								{
									"display_url": "developer.twitter.com/en/community",
									"expanded_url": "https://developer.twitter.com/en/community",
									"indices": [
										0,
										23
									],
									"url": "https://t.co/3ZX3TNiZCY"
								}
							]
						}
					},
					"favourites_count": 2112,
					"follow_request_sent": null,
					"followers_count": 536197,
					"friends_count": 2018,
					"geo_enabled": true,
					"id": 2244994945,
					"is_translator": false,
					"lang": null,
					"listed_count": 1950,
					"location": "127.0.0.1",
					"name": "Twitter Dev",
					"profile_background_color": "FFFFFF",
					"profile_background_image_url": "http://abs.twimg.com/images/themes/theme1/bg.png",
					"profile_background_image_url_https": "https://abs.twimg.com/images/themes/theme1/bg.png",
					"profile_background_tile": false,
					"profile_banner_url": "https://pbs.twimg.com/profile_banners/2244994945/1633532194",
					"profile_image_url": "http://pbs.twimg.com/profile_images/1445764922474827784/W2zEPN7U_normal.jpg",
					"profile_image_url_https": "https://pbs.twimg.com/profile_images/1445764922474827784/W2zEPN7U_normal.jpg",
					"profile_link_color": "0084B4",
					"profile_sidebar_border_color": "FFFFFF",
					"profile_sidebar_fill_color": "DDEEF6",
					"profile_text_color": "333333",
					"profile_use_background_image": false,
					"protected": false,
					"screen_name": "TwitterDev",
					"show_all_inline_media": null,
					"status": null,
					"statuses_count": 3881,
					"time_zone": null,
					"url": "https://t.co/3ZX3TNiZCY",
					"utc_offset": null,
					"verified": true,
					"withheld_in_countries": [],
					"withheld_scope": null
				},
				"withheld_copyright": false,
				"withheld_in_countries": null,
				"withheld_scope": null
			}
		}));

		await page.reload();

		await Promise.all([
			await page.route(`/proxy/twitter/like/1228393702244134912`, route => route.fulfill()),
			await page.click('.article .likeButton'),
		]);
	});

	test('retweet feedback', async ({page}) => {
		await page.goto('/');
		await page.mainFrame().evaluate(() => {
			window.localStorage.setItem('SoshalThingYew Timelines', JSON.stringify([{
				"title": "Timeline",
				"endpoints": [
					{
						"service": "Twitter",
						"endpoint_type": 4,
						"params": { "id": "1228393702244134912" },
						"on_start": true,
						"on_refresh": false
					}
				]
			}]));
		});

		await page.route('/proxy/twitter/status/', route => route.fulfill({
			body: {
				"coordinates": null,
				"created_at": "Fri Feb 14 19:00:55 +0000 2020",
				"current_user_retweet": null,
				"display_text_range": [
					0,
					97
				],
				"entities": {
					"hashtags": [],
					"symbols": [],
					"urls": [],
					"user_mentions": [],
					"media": null
				},
				"extended_entities": null,
				"favorite_count": 402,
				"favorited": false,
				"filter_level": null,
				"id": 1228393702244135000,
				"in_reply_to_user_id": null,
				"in_reply_to_screen_name": null,
				"in_reply_to_status_id": null,
				"lang": "en",
				"place": null,
				"possibly_sensitive": null,
				"quoted_status_id": null,
				"quoted_status": null,
				"retweet_count": 112,
				"retweeted": false,
				"retweeted_status": null,
				"source": {
					"name": "Twitter Web App",
					"url": "https://mobile.twitter.com"
				},
				"text": "What did the developer write in their Valentine’s card?\n  \nwhile(true) {\n    I = Love(You);  \n}",
				"truncated": false,
				"user": {
					"contributors_enabled": false,
					"created_at": "2013-12-14T04:35:55Z",
					"default_profile": false,
					"default_profile_image": false,
					"description": "The voice of the #TwitterDev team and your official source for updates, news, and events, related to the #TwitterAPI.",
					"entities": {
						"description": {
							"urls": []
						},
						"url": {
							"urls": [
								{
									"display_url": "developer.twitter.com/en/community",
									"expanded_url": "https://developer.twitter.com/en/community",
									"indices": [
										0,
										23
									],
									"url": "https://t.co/3ZX3TNiZCY"
								}
							]
						}
					},
					"favourites_count": 2112,
					"follow_request_sent": null,
					"followers_count": 536197,
					"friends_count": 2018,
					"geo_enabled": true,
					"id": 2244994945,
					"is_translator": false,
					"lang": null,
					"listed_count": 1950,
					"location": "127.0.0.1",
					"name": "Twitter Dev",
					"profile_background_color": "FFFFFF",
					"profile_background_image_url": "http://abs.twimg.com/images/themes/theme1/bg.png",
					"profile_background_image_url_https": "https://abs.twimg.com/images/themes/theme1/bg.png",
					"profile_background_tile": false,
					"profile_banner_url": "https://pbs.twimg.com/profile_banners/2244994945/1633532194",
					"profile_image_url": "http://pbs.twimg.com/profile_images/1445764922474827784/W2zEPN7U_normal.jpg",
					"profile_image_url_https": "https://pbs.twimg.com/profile_images/1445764922474827784/W2zEPN7U_normal.jpg",
					"profile_link_color": "0084B4",
					"profile_sidebar_border_color": "FFFFFF",
					"profile_sidebar_fill_color": "DDEEF6",
					"profile_text_color": "333333",
					"profile_use_background_image": false,
					"protected": false,
					"screen_name": "TwitterDev",
					"show_all_inline_media": null,
					"status": null,
					"statuses_count": 3881,
					"time_zone": null,
					"url": "https://t.co/3ZX3TNiZCY",
					"utc_offset": null,
					"verified": true,
					"withheld_in_countries": [],
					"withheld_scope": null
				},
				"withheld_copyright": false,
				"withheld_in_countries": null,
				"withheld_scope": null
			}
		}));

		await page.reload();

		await Promise.all([
			await page.route(`/proxy/twitter/retweet/1228393702244134912`, route => route.fulfill()),
			await page.click('.article .repostButton'),
		]);
	});
})