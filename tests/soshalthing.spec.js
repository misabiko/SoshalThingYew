const {expect, test} = require('@playwright/test');

test('visits the app root url', async ({page}) => {
	await page.goto('/');
});

test('timeline without endpoint', async ({page}) => {
	await page.goto('/');
	await page.mainFrame().evaluate(() => {
		window.localStorage.setItem('SoshalThingYew Timelines', JSON.stringify([
			{"title": "Timeline"}
		]));
	});
	await page.reload();

	await expect(page.locator('.timeline')).toHaveCount(1);

	await page.click("#sidebarButtons button[title = 'Expand sidebar']");

	await expect(page.locator("#sidebar > .sidebarMenu > div.box").nth(2)).toBeEmpty();
});

test.describe('main timeline', () => {
	test('via search param', async ({page}) => {
		await page.goto('/');
		await page.mainFrame().evaluate(() => {
			window.localStorage.setItem('SoshalThingYew Timelines', JSON.stringify([
				{"title": "Timeline"}
			]));
		});
		await page.goto('/?single_timeline=true');

		await expect(page.locator('.timeline').first()).toHaveClass(/mainTimeline/);
	});

	test('via local storage', async ({page}) => {
		await page.goto('/');
		await page.mainFrame().evaluate(() => {
			window.localStorage.setItem('SoshalThingYew', JSON.stringify({
				display_mode: {
					type: "Single",
					container: "Column",
					column_count: 1,
				}
			}));
			window.localStorage.setItem('SoshalThingYew Timelines', JSON.stringify([
				{"title": "Timeline"}
			]));
		});
		await page.reload();

		await expect(page.locator('.timeline').first()).toHaveClass(/mainTimeline/);
	});

	test('setting main timeline retains order', async ({page}) => {
		await page.goto('/');
		await page.mainFrame().evaluate(() => {
			window.localStorage.setItem('SoshalThingYew Timelines', JSON.stringify([
				{"title": "Timeline1"},
				{"title": "Timeline2"},
				{"title": "Timeline3"},
			]));
		});
		await page.reload();

		await page.click('.timeline:nth-child(2) .timelineHeader .timelineButtons button[title = "Expand options"]');

		await page.click('button:has-text("Set as main timeline")');

		await page.click('#sidebarButtons button[title = "Multiple Timelines"]');

		for (let i = 1; i <= 3; ++i)
			await expect(page.locator(`.timeline:nth-child(${i}) .timelineLeftHeader > strong`))
				.toHaveText('Timeline' + i, {timeout: 500});
	});

	test('removing main timeline retains order', async ({page}) => {
		await page.goto('/');
		await page.mainFrame().evaluate(() => {
			window.localStorage.setItem('SoshalThingYew Timelines', JSON.stringify([
				{"title": "Timeline1"},
				{"title": "Timeline2"},
				{"title": "Timeline3"},
			]));
		});
		await page.reload();

		await page.click('.timeline:nth-child(2) .timelineHeader .timelineButtons button[title = "Expand options"]');

		await page.click('button:has-text("Set as main timeline")');

		await page.click('text=Remove timeline');

		await page.click('#sidebarButtons button[title = "Multiple Timelines"]');

		await expect(page.locator('.timeline:nth-child(1) .timelineLeftHeader > strong'))
			.toHaveText('Timeline1', {timeout: 500});
		await expect(page.locator('.timeline:nth-child(2) .timelineLeftHeader > strong'))
			.toHaveText('Timeline3', {timeout: 500});
	});
});

test.describe('article actions', () => {
	test('like feedback', async ({page}) => {
		await page.goto('/');
		await page.mainFrame().evaluate(() => {
			window.localStorage.setItem('SoshalThingYew Timelines', JSON.stringify([{
				"title": "Timeline",
				"endpoints": [
					{
						"service": "Dummy",
						"endpoint_type": 0,
						"params": {},
						"on_start": true,
						"on_refresh": true
					}
				]
			}]));
		});
		await page.reload();

		await page.click('.article .likeButton');

		await expect(page.locator('.article .likeButton')).toHaveClass(/likedPostButton/);
	});

	test('repost feedback', async ({page}) => {
		await page.goto('/');
		await page.mainFrame().evaluate(() => {
			window.localStorage.setItem('SoshalThingYew Timelines', JSON.stringify([{
				"title": "Timeline",
				"endpoints": [
					{
						"service": "Dummy",
						"endpoint_type": 0,
						"params": {},
						"on_start": true,
						"on_refresh": true
					}
				]
			}]));
		});
		await page.reload();

		await page.click('.article .repostButton');

		await expect(page.locator('.article .repostButton')).toHaveClass(/repostedPostButton/);
	});
})