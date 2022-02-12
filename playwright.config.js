const config = {
	webServer: {
		command: 'cargo run -p utils --bin server',
		port: 8080,
		timeout: 10 * 1000,
		reuseExistingServer: !process.env.CI,
	},
	use: {
		baseURL: 'http://localhost:8080/',
	},
	reporter: process.env.CI ? 'github' : 'list',
};
module.exports = config;