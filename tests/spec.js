const {expect, test} = require('@playwright/test');

test.describe('SoshalThing', () => {
	test('visits the app root url', async ({page}) => {
		await page.goto('/');
	});

	test('timeline without endpoint', async ({page}) => {
		await page.goto('/');
		await page.mainFrame().evaluate(() => {
			window.localStorage.setItem('SoshalThingYew', JSON.stringify({
				display_mode: {
					type: "Single",
				}
			}));
			window.localStorage.setItem('SoshalThingYew Timelines', JSON.stringify([
				{"title": "Home"}
			]));
		});
		await page.reload();

		await expect(page.locator('.timeline')).toHaveCount(1);

		await page.click("#sidebarButtons button[title = 'Expand sidebar']");

		await expect(page.locator("#sidebar > .sidebarMenu > div.box").nth(2)).toBeEmpty();
	});

	test('main timeline search param', async ({page}) => {
		await page.goto('/');
		await page.mainFrame().evaluate(() => {
			window.localStorage.setItem('SoshalThingYew Timelines', JSON.stringify([
				{"title": "Home"}
			]));
		});
		await page.goto('/?single_timeline=true');

		await expect(page.locator('.timeline').first()).toHaveClass(/mainTimeline/);
	});

	test('main timeline storage', async ({page}) => {
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
				{"title": "Home"}
			]));
		});
		await page.reload();

		await expect(page.locator('.timeline').first()).toHaveClass(/mainTimeline/);
	});

	test.skip('repost feedback', async ({page}) => {
		await page.goto('/');
		await page.mainFrame().evaluate(() => {
			window.localStorage.setItem('SoshalThingYew', JSON.stringify({
				display_mode: {
					type: "Single",
					container: "Column",
					column_count: 1,
				}
			}));
			window.localStorage.setItem('SoshalThingYew Timelines', JSON.stringify([{
				"title": "Home",
				"endpoints": [
					{
						"service": "Dummy Service",
						"endpoint_type": 0,
						"params": {},
						"on_start": true,
						"on_refresh": true
					}
				]
			}]));
		});
		await page.reload();
	});
});