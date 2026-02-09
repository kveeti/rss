import { useSearchParams } from "@solidjs/router";
import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { For, Match, Show, Switch, createEffect } from "solid-js";

import { Button } from "../components/button";
import { Empty } from "../components/empty";
import { Entry } from "../components/entry";
import { NavPaginationLinks, Pagination, buildPaginatedHref } from "../components/pagination";
import { DefaultNavLinks, Nav, NavWrap, Page } from "../layout";
import { type UnreadEntriesParams, unreadEntriesQueryOptions } from "./unread-page.data";

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
					<EntriesList />
				</main>
			</Page>
		</>
	);
}

function useUnreadEntriesQuery(params: () => UnreadEntriesParams) {
	const queryClient = useQueryClient();

	const query = createQuery(() => unreadEntriesQueryOptions(params()));

	createEffect(() => {
		const data = query.data;
		if (!data) return;

		const nextId = data.next_id;
		const prevId = data.prev_id;

		if (nextId) {
			queryClient.prefetchQuery(
				unreadEntriesQueryOptions({ leftId: undefined, rightId: nextId })
			);
		}

		if (prevId) {
			queryClient.prefetchQuery(
				unreadEntriesQueryOptions({ leftId: prevId, rightId: undefined })
			);
		}
	});

	return query;
}

function EntriesList() {
	const [searchParams] = useSearchParams();

	const params = () => ({
		leftId: searchParams.left as string | undefined,
		rightId: searchParams.right as string | undefined,
	});

	const query = useUnreadEntriesQuery(params);

	const nextHref = () =>
		buildPaginatedHref("right", query.data?.next_id, "/unread", searchParams);
	const prevHref = () => buildPaginatedHref("left", query.data?.prev_id, "/unread", searchParams);

	return (
		<Switch>
			<Match when={query.isError}>
				<div class="space-y-4">
					<p class="bg-red-a4 p-4">
						{query.error instanceof Error
							? query.error.message
							: "Error loading entries"}
					</p>

					<Button onClick={() => query.refetch()}>Retry</Button>
				</div>
			</Match>

			<Match when={query.isLoading}>
				<EntriesSkeleton />
			</Match>

			<Match when={query.data?.entries.length} fallback={<Empty>No unread entries</Empty>}>
				<ul class="divide-gray-a3 -mx-3 mb-40 divide-y">
					<For each={query.data?.entries}>
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
			</Match>
		</Switch>
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
	const [searchParams] = useSearchParams();

	const params = () => ({
		leftId: searchParams.left as string | undefined,
		rightId: searchParams.right as string | undefined,
	});

	const query = useUnreadEntriesQuery(params);

	const nextHref = () =>
		buildPaginatedHref("right", query.data?.next_id, "/unread", searchParams);
	const prevHref = () => buildPaginatedHref("left", query.data?.prev_id, "/unread", searchParams);

	return (
		<Show when={!query.isLoading && !query.isError} fallback={<NavPaginationLinks />}>
			<NavPaginationLinks nextHref={nextHref()} prevHref={prevHref()} />
		</Show>
	);
}
