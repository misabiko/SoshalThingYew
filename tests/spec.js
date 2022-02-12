const {expect, test} = require('@playwright/test');

test.describe('SoshalThing', () => {
	test('visits the app root url', async ({page}) => {
		await page.goto('/');
	});

	test.describe('', () => {
		test.use({
			storageState: {
				origins: [{
					origin: 'http://localhost:8080',
					localStorage: [
						{
							name: 'SoshalThingYew',
							value: JSON.stringify({
								display_mode: {
									type: "Single",
								}
							})
						},
						{
							name: 'SoshalThingYew Timelines',
							value: JSON.stringify([
								{"title": "Home"}
							])
						}
					]
				}]
			}
		});
		test('timeline without endpoint', async ({page}) => {
			await page.goto('/');

			await expect(page.locator('.timeline')).toHaveCount(1);

			await page.click("#sidebarButtons button[title = 'Expand sidebar']");

			await expect(page.locator("#sidebar > .sidebarMenu > div.box").nth(2)).toBeEmpty();
		});
	});

	test.describe('', () => {
		test.use({
			storageState: {
				origins: [{
					origin: 'http://localhost:8080',
					localStorage: [
						{
							name: 'SoshalThingYew Timelines',
							value: JSON.stringify([
								{"title": "Home"}
							])
						}
					]
				}]
			}
		});
		test('main timeline search param', async ({page}) => {
			await page.goto('/?single_timeline=true');

			await expect(page.locator('.timeline').first()).toHaveClass(/mainTimeline/);
		});
	});

	test.describe('', () => {
		test.use({
			storageState: {
				origins: [{
					origin: 'http://localhost:8080',
					localStorage: [
						{
							name: 'SoshalThingYew',
							value: JSON.stringify({
								display_mode: {
									type: "Single",
									container: "Column",
									column_count: 1,
								}
							})
						},
						{
							name: 'SoshalThingYew Timelines',
							value: JSON.stringify([
								{"title": "Home"}
							])
						}
					]
				}]
			}
		});
		test('main timeline storage', async ({page}) => {
			await page.goto('/');

			await expect(page.locator('.timeline').first()).toHaveClass(/mainTimeline/);
		});
	});

	test.describe('', () => {
		test.use({
			storageState: {
				origins: [{
					origin: 'http://localhost:8080',
					localStorage: [
						{
							name: 'SoshalThingYew',
							value: JSON.stringify({
								display_mode: {
									type: "Single",
									container: "Column",
									column_count: 1,
								}
							})
						},
						{
							name: 'SoshalThingYew Timelines',
							value: JSON.stringify([
								{
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
								}
							])
						}
					]
				}]
			}
		});
		test.skip('repost feedback', async ({page}) => {
			await page.goto('/');
		});
	});
});