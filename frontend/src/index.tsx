/* @refresh reload */
import { Navigate, type RouteDefinition, Router } from "@solidjs/router";
import "solid-devtools";
import { lazy } from "solid-js";
import { render } from "solid-js/web";

import "./styles.css";

const root = document.getElementById("root");

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
	throw new Error(
		"Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?"
	);
}

export const routes: RouteDefinition[] = [
	{
		path: "/",
		component: lazy(() => import("./pages/index-page")),
	},
	{
		path: "**",
		component: () => <Navigate href="/" />,
	},
];

render(() => <Router root={(props) => props.children}>{routes}</Router>, root!);
