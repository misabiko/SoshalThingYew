const config = {
	webServer: {
		command: 'cargo run -p utils --bin server',
		port: 8080,
		timeout: 10 * 1000,
		reuseExistingServer: !process.env.CI,
	},
	use: {
		baseURL: 'http://localhost:8080/',
		trace: 'on-first-retry',
	},
	reporter: process.env.CI ? 'github' : 'list',
	retries: 1,
};
module.exports = config;