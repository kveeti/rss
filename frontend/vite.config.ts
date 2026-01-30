import devtools from "solid-devtools/vite";
import { defineConfig } from "vite";
import { VitePWA } from "vite-plugin-pwa";
import solidPlugin from "vite-plugin-solid";

export default defineConfig({
	plugins: [
		devtools(),
		solidPlugin(),
		VitePWA({
			registerType: "prompt",
			injectRegister: "inline",
			devOptions: {
				enabled: true,
			},
			workbox: {
				globPatterns: ["**/*.{js,css,html,ico,png,svg,woff,woff2,ttf,eot}"],
			},
		}),
	],
	build: {
		target: "esnext",
	},
	clearScreen: false,
	server: { port: 3000 },
	preview: { port: 3000 },
});
