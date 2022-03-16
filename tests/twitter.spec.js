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
		await page.reload();

		await Promise.all([
			await page.route(`/proxy/twitter/retweet/1228393702244134912`, route => route.fulfill()),
			await page.click('.article .repostButton'),
		]);
	});
})