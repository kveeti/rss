/* @refresh reload */
import { Navigate, type RouteDefinition, Router } from "@solidjs/router";
import "solid-devtools";
import { Suspense, lazy } from "solid-js";
import { render } from "solid-js/web";

import { NavPaginationLinks } from "./components/pagination";
import { DefaultNavLinks, Nav, NavWrap } from "./layout";
import { preloadsEntriesPage } from "./pages/entries-page.data";
import { preloadsFeedEditPage } from "./pages/feed-edit-page.data";
import { preloadsFeedPage } from "./pages/feed-page.data";
import { preloadsFeedsPage } from "./pages/feeds-page.data";
import { preloadsNewFeedPage } from "./pages/new-feed-page.data";
import { preloadsUnreadPage } from "./pages/unread-page.data";
import "./styles.css";

const root = document.getElementById("root");

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
	throw new Error(
		"Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?"
	);
}

export const routes: RouteDefinition[] = [
	{
		path: "/feeds",
		component: () => (
			<Suspense
				fallback={
					<NavWrap>
						<Nav>
							<DefaultNavLinks />
						</Nav>
					</NavWrap>
				}
			>
				{lazy(() => import("./pages/feeds-page"))()}
			</Suspense>
		),
		preload: preloadsFeedsPage,
	},
	{
		path: "/feeds/new",
		component: () => (
			<Suspense
				fallback={
					<NavWrap>
						<Nav>
							<DefaultNavLinks />
						</Nav>
					</NavWrap>
				}
			>
				{lazy(() => import("./pages/new-feed-page"))()}
			</Suspense>
		),
		preload: preloadsNewFeedPage,
	},
	{
		path: "/feeds/:feedId",
		component: () => (
			<Suspense
				fallback={
					<NavWrap>
						<Nav>
							<DefaultNavLinks />
						</Nav>
					</NavWrap>
				}
			>
				{lazy(() => import("./pages/feed-page"))()}
			</Suspense>
		),
		preload: ({ params }) => preloadsFeedPage(params.feedId),
	},
	{
		path: "/feeds/:feedId/edit",
		component: () => (
			<Suspense
				fallback={
					<NavWrap>
						<Nav>
							<DefaultNavLinks />
						</Nav>
					</NavWrap>
				}
			>
				{lazy(() => import("./pages/feed-edit-page"))()}
			</Suspense>
		),
		preload: ({ params }) => preloadsFeedEditPage(params.feedId),
	},
	{
		path: "/unread",
		component: () => (
			<Suspense
				fallback={
					<NavWrap>
						<Nav>
							<div class="flex w-full justify-between">
								<DefaultNavLinks />
								<NavPaginationLinks />
							</div>
						</Nav>
					</NavWrap>
				}
			>
				{lazy(() => import("./pages/unread-page"))()}
			</Suspense>
		),
		preload: ({ location }) => preloadsUnreadPage({ search: location.search }),
	},
	{
		path: "/entries",
		component: () => (
			<Suspense
				fallback={
					<NavWrap>
						<Nav>
							<div class="flex w-full justify-between">
								<DefaultNavLinks />
								<NavPaginationLinks />
							</div>
						</Nav>
					</NavWrap>
				}
			>
				{lazy(() => import("./pages/entries-page"))()}
			</Suspense>
		),
		preload: ({ location }) => preloadsEntriesPage({ search: location.search }),
	},
	{
		path: "**",
		component: () => <Navigate href="/feeds" />,
	},
];

render(() => <Router root={(props) => props.children}>{routes}</Router>, root!);
