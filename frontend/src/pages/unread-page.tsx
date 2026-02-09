import { createAsync, revalidate, useSearchParams } from "@solidjs/router";
import { For, Show, createEffect, resetErrorBoundaries } from "solid-js";

import { Boundaries } from "../components/boundaries";
import { Button } from "../components/button";
import { Empty } from "../components/empty";
import { Entry } from "../components/entry";
import { NavPaginationLinks, Pagination, buildPaginatedHref } from "../components/pagination";
import { DefaultNavLinks, Nav, NavWrap, Page } from "../layout";
import { queryEntries } from "./entries-page.data";
import { getUnreadEntries } from "./unread-page.data";

export default function UnreadPage() {
	return (
		<>
			<NavWrap>
				<Nav>
					<DefaultNavLinks />
					<NavPagination />
				</Nav>
			</NavWrap>

			<Page>
				<main class="mx-auto max-w-160 px-3">
					<Boundaries
						loading={<EntriesSkeleton />}
						error={(_reset) => (
							<div class={"space-y-4"}>
								<p class="bg-red-a4 p-4">Error loading entries</p>

								<Button
									onClick={() => {
										revalidate(queryEntries.key);
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
						<EntriesList />
					</Boundaries>
				</main>
			</Page>
		</>
	);
}

function EntriesList() {
	const [searchParams] = useSearchParams();

	const entriesCursor = createAsync(() =>
		getUnreadEntries(
			searchParams.left as string | undefined,
			searchParams.right as string | undefined
		)
	);

	createEffect(() => {
		const nextId = entriesCursor()?.next_id;
		const prevId = entriesCursor()?.prev_id;

		if (nextId) {
			queryEntries({ right: nextId, unread: "true" });
		}

		if (prevId) {
			queryEntries({ left: prevId, unread: "true" });
		}
	});

	const nextHref = () =>
		buildPaginatedHref("right", entriesCursor()?.next_id, "/unread", searchParams);
	const prevHref = () =>
		buildPaginatedHref("left", entriesCursor()?.prev_id, "/unread", searchParams);

	return (
		<>
			{!entriesCursor()?.entries.length ? (
				<Empty>No unread entries</Empty>
			) : (
				<>
					<ul class="divide-gray-a3 -mx-3 mb-40 divide-y">
						<For each={entriesCursor()?.entries}>
							{(entry) => (
								<Entry.Root entry={entry}>
									<div class="flex gap-3">
										<Entry.Icon />
										<Entry.Content>
											<Entry.Title />
											<Entry.Meta>
												<Entry.Date />
												<Entry.Comments />
												<Entry.ReadToggle />
											</Entry.Meta>
										</Entry.Content>
									</div>
								</Entry.Root>
							)}
						</For>
					</ul>

					<div class="pwa:bottom-28 pointer-events-none fixed right-0 bottom-13 left-0 sm:bottom-0">
						<div class="mx-auto flex max-w-160 justify-end">
							<Pagination prevHref={prevHref()} nextHref={nextHref()} />
						</div>
					</div>
				</>
			)}
		</>
	);
}

function EntriesSkeleton() {
	return (
		<ul class="space-y-4" aria-hidden="true">
			{Array.from({ length: 7 }).map(() => (
				<li class="bg-gray-a2/20 flex w-full flex-col gap-2 p-4">
					<div class="flex items-center gap-3">
						<div class="inline-flex size-6"></div>

						<p class="invisible">0</p>
					</div>

					<p class="invisible">0</p>
				</li>
			))}
		</ul>
	);
}

function NavPagination() {
	return (
		<Boundaries loading={<NavPaginationLinks />} error={(_reset) => <NavPaginationLinks />}>
			<NavPaginationInner />
		</Boundaries>
	);
}

function NavPaginationInner() {
	const [searchParams] = useSearchParams();

	const entriesCursor = createAsync(() =>
		getUnreadEntries(
			searchParams.left as string | undefined,
			searchParams.right as string | undefined
		)
	);

	const nextHref = () =>
		buildPaginatedHref("right", entriesCursor()?.next_id, "/unread", searchParams);
	const prevHref = () =>
		buildPaginatedHref("left", entriesCursor()?.prev_id, "/unread", searchParams);

	return <NavPaginationLinks nextHref={nextHref()} prevHref={prevHref()} />;
}
