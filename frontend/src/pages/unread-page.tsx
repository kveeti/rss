import { createAsync, revalidate, useSearchParams } from "@solidjs/router";
import { ErrorBoundary, Suspense, resetErrorBoundaries } from "solid-js";

import { Button } from "../components/button";
import { Entry } from "../components/entry";
import { NavPaginationLinks, Pagination, buildPaginatedHref } from "../components/pagination";
import { DefaultNavLinks, Nav, NavWrap, Page } from "../layout";
import { queryEntries } from "./entries-page.data";

type UnreadEntriesParams = {
	right?: string;
	left?: string;
};

export default function UnreadPage() {
	const [searchParams] = useSearchParams();

	const unreadEntriesParams = () => ({
		right: searchParams.right as string | undefined,
		left: searchParams.left as string | undefined,
	});

	return (
		<>
			<NavWrap>
				<Nav>
					<div class="flex w-full justify-between">
						<DefaultNavLinks />
						<NavPagination {...unreadEntriesParams()} />
					</div>
				</Nav>
			</NavWrap>

			<Page>
				<main class="mx-auto max-w-160 px-3">
					<Suspense fallback={<EntriesSkeleton />}>
						<ErrorBoundary
							fallback={(_error, _reset) => (
								<div class={"space-y-4"}>
									<p class="bg-red-a4 p-4">Error loading entries</p>

									<Button
										onClick={() => {
											revalidate(queryEntries.keyFor(unreadEntriesParams()));
											// Reset all error boundaries here so that
											// the one in nav also get reset
											resetErrorBoundaries();
										}}
									>
										Retry
									</Button>
								</div>
							)}
						>
							<EntriesList {...unreadEntriesParams()} />
						</ErrorBoundary>
					</Suspense>
				</main>
			</Page>
		</>
	);
}

function EntriesList(props: UnreadEntriesParams) {
	const entriesCursor = createAsync(() => queryEntries({ ...props, unread: "true" }));

	const [searchParams] = useSearchParams();

	const nextHref = () => buildPaginatedHref("right", entriesCursor()?.next_id, searchParams);
	const prevHref = () => buildPaginatedHref("left", entriesCursor()?.prev_id, searchParams);

	return (
		<>
			<ul class="divide-gray-a3 -mx-3 mb-40 divide-y">
				{entriesCursor()?.entries?.map((entry) => {
					const dateStr = entry.published_at || entry.entry_updated_at;
					const date = dateStr ? new Date(dateStr) : undefined;

					return (
						<Entry
							feedId={entry.feed_id}
							hasIcon={entry.has_icon}
							title={entry.title}
							date={date}
							commentsUrl={entry.comments_url}
							url={entry.url}
						/>
					);
				})}
			</ul>

			<div class="pwa:bottom-28 pointer-events-none fixed right-0 bottom-13 left-0 sm:bottom-0">
				<div class="mx-auto flex max-w-160 justify-end">
					<Pagination prevHref={prevHref()} nextHref={nextHref()} />
				</div>
			</div>
		</>
	);
}

function EntriesSkeleton() {
	return (
		<main class="mx-auto max-w-160 px-3">
			<ul class="space-y-4" aria-hidden="true">
				{Array.from({ length: 14 }).map(() => (
					<li class="bg-gray-a2/20 flex w-full flex-col gap-2 p-4">
						<div class="flex items-center gap-3">
							<div class="inline-flex size-6"></div>
							<p class="invisible">0</p>
						</div>
						<p class="invisible">0</p>
					</li>
				))}
			</ul>
		</main>
	);
}

function NavPagination(props: UnreadEntriesParams) {
	return (
		<ErrorBoundary fallback={<NavPaginationLinks />}>
			<Suspense fallback={<NavPaginationLinks />}>
				<NavPaginationInner {...props} />
			</Suspense>
		</ErrorBoundary>
	);
}

function NavPaginationInner(props: UnreadEntriesParams) {
	const entriesCursor = createAsync(() => queryEntries(props));
	const [searchParams] = useSearchParams();

	const nextHref = () =>
		buildPaginatedHref("right", entriesCursor()?.next_id, "/unread", searchParams);
	const prevHref = () =>
		buildPaginatedHref("left", entriesCursor()?.prev_id, "/unread", searchParams);

	return <NavPaginationLinks nextHref={nextHref()} prevHref={prevHref()} />;
}
