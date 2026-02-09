/* @refresh reload */
import { Navigate, type RouteDefinition, Router } from "@solidjs/router";
import { QueryClient, QueryClientProvider } from "@tanstack/solid-query";
import { SolidQueryDevtools } from "@tanstack/solid-query-devtools";
import "solid-devtools";
import { Show, Suspense, createSignal, lazy } from "solid-js";
import { render } from "solid-js/web";
import { registerSW } from "virtual:pwa-register";

import { Button } from "./components/button";
import { NavPaginationLinks } from "./components/pagination";
import { DefaultNavLinks, Nav, NavWrap } from "./layout";
import { prefetchEntriesPage } from "./pages/entries-page.data";
import { prefetchFeedEditPage } from "./pages/feed-edit-page.data";
import { prefetchFeedPage } from "./pages/feed-page.data";
import { prefetchFeedsPage } from "./pages/feeds-page.data";
import { preloadsNewFeedPage } from "./pages/new-feed-page.data";
import { prefetchUnreadPage } from "./pages/unread-page.data";
import "./styles.css";

const root = document.getElementById("root");

const [showUpdatePopup, setShowUpdatePopup] = createSignal(false);
const updateSW = registerSW({
	onNeedRefresh() {
		setShowUpdatePopup(true);
	},
});

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
		preload: () => prefetchFeedsPage(queryClient),
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
		preload: ({ params }) => prefetchFeedPage(queryClient, params.feedId),
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
		preload: ({ params }) => prefetchFeedEditPage(queryClient, params.feedId),
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
		preload: ({ location }) => prefetchUnreadPage(queryClient, { search: location.search }),
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
		preload: ({ location }) => prefetchEntriesPage(queryClient, { search: location.search }),
	},
	{
		path: "**",
		component: () => <Navigate href="/feeds" />,
	},
];

const queryClient = new QueryClient();

render(
	() => (
		<QueryClientProvider client={queryClient}>
			<SolidQueryDevtools />

			<Router
				root={(props) => (
					<>
						{props.children}
						<Show when={showUpdatePopup()}>
							<div class="fixed right-0 bottom-0 z-50 p-4">
								<div class="bg-gray-2 border-gray-a5 border p-3">
									<p class="text-gray-12">
										A new version is available. Click refresh to update
									</p>
									<div class="mt-3 flex justify-end gap-2">
										<Button
											variant="ghost"
											onClick={() => setShowUpdatePopup(false)}
										>
											Close
										</Button>
										<Button onClick={() => updateSW()}>Refresh</Button>
									</div>
								</div>
							</div>
						</Show>
					</>
				)}
			>
				{routes}
			</Router>
		</QueryClientProvider>
	),
	root!
);
