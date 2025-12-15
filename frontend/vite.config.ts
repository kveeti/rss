import devtools from "solid-devtools/vite";
import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";

export default defineConfig({
	plugins: [devtools(), solidPlugin()],
	build: {
		target: "esnext",
	},
	clearScreen: false,
	server: { port: 3000 },
	preview: { port: 3000 },
});
