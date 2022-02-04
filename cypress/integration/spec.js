describe('SoshalThing', () => {
	it('visits the app root url', () => {
		cy.visit('/');
	})

	it('timeline without endpoint', () => {
		cy.session("timeline without endpoint", () => {
			window.localStorage.setItem("SoshalThingYew", JSON.stringify({
				display_mode: {
					type: "Single",
				}
			}));
			window.localStorage.setItem("SoshalThingYew Timelines", JSON.stringify([
				{
					"title": "Home"
				}
			]));
		});
		cy.visit('/');

		cy.get(".timeline")

		cy.get("#sidebarButtons button[title = 'Expand sidebar']").click()

		cy.get("#sidebar > .sidebarMenu > div.box").eq(2).should("be.empty")
	})

	it('main timeline search param', () => {
		cy.session("main timeline search param", () => {
			window.localStorage.setItem("SoshalThingYew Timelines", JSON.stringify([
				{
					"title": "Home"
				}
			]));
		});
		cy.visit('/?single_timeline=true');

		cy.get(".timeline.mainTimeline")
	})

	it('main timeline storage', () => {
		cy.session("main timeline storage", () => {
			window.localStorage.setItem("SoshalThingYew", JSON.stringify({
				display_mode: {
					type: "Single",
					container: "Column",
					column_count: 1,
				}
			}));
			window.localStorage.setItem("SoshalThingYew Timelines", JSON.stringify([
				{
					"title": "Home"
				}
			]));
		});
		cy.visit('/');

		cy.get(".timeline.mainTimeline")
	})

	it.skip('repost feedback', () => {
		cy.session("repost feedback", () => {
			window.localStorage.setItem("SoshalThingYew Timelines", JSON.stringify([
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
			]));
		});
		cy.visit('/');
	})
})