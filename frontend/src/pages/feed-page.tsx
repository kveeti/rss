import { useParams, useSearchParams } from "@solidjs/router";
import { createQuery } from "@tanstack/solid-query";
import { For, Match, Switch } from "solid-js";

import { Button, buttonStyles } from "../components/button";
import { Empty } from "../components/empty";
import { Entry } from "../components/entry";
import { FeedIcon } from "../components/feed-icon";
import { IconSettings } from "../components/icons/settings";
import { Pagination } from "../components/pagination";
import { DefaultNavLinks, Nav, NavWrap, Page } from "../layout";
import { feedEntriesQueryOptions, feedQueryOptions } from "./feed-page.data";

export default function FeedPage() {
	const params = useParams();
	const feedId = params.feedId;
	if (!feedId) {
		throw new Error("feedId is required");
	}

	return (
		<>
			<NavWrap>
				<Nav>
					<DefaultNavLinks />
				</Nav>
			</NavWrap>

			<Page>
				<main class="mx-auto max-w-160 px-3">
					<FeedDetails feedId={feedId} />
					<FeedEntries feedId={feedId} />
				</main>
			</Page>
		</>
	);
}

function FeedDetails(props: { feedId: string }) {
	const query = createQuery(() => feedQueryOptions(props.feedId));

	return (
		<Switch>
			<Match when={query.isError}>
				<FeedDetailsError
					class="mt-4"
					retry={() => {
						query.refetch();
					}}
				/>
			</Match>

			<Match when={query.isLoading}>
				<FeedDetailsSkeleton />
			</Match>

			<Match when={query.data} keyed>
				{(feed) => (
					<div class="mx-auto my-4 flex w-full justify-between gap-6">
						<div class="font-cool relative text-xl">
							<FeedIcon
								class="me-2.5 inline size-5.5 align-text-bottom"
								feedId={feed.id}
								hasIcon={feed.has_icon}
								fallbackUrl={feed.feed_url}
							/>
							<h1 class="inline font-medium">{feed.title}</h1>

							<a href={feed.site_url ?? feed.feed_url} class="absolute inset-0">
								<span class="sr-only">{feed.title}</span>
							</a>
						</div>

						<a
							href={`/feeds/${props.feedId}/edit`}
							class={buttonStyles({ variant: "ghost", size: "withIcon" }) + " gap-3"}
						>
							<IconSettings /> <span>Edit</span>
						</a>
					</div>
				)}
			</Match>
		</Switch>
	);
}

function FeedDetailsError(props: { class?: string; retry: () => void }) {
	return (
		<div class={"space-y-4" + (props.class ? ` ${props.class}` : "")}>
			<p class="bg-red-a4 p-4">Error loading feed details</p>

			<Button onClick={props.retry}>Retry</Button>
		</div>
	);
}

function FeedDetailsSkeleton() {
	return (
		<div aria-hidden="true" class="my-4 flex items-center gap-4">
			<div class="bg-gray-a2/40 size-6" />
			<h1 class="bg-gray-a2/40 w-[40%] text-3xl leading-none">
				<span class="invisible">0</span>
			</h1>
		</div>
	);
}

function FeedEntries(props: { feedId: string }) {
	const [searchParams] = useSearchParams();

	return (
		<FeedEntriesList
			feedId={props.feedId}
			limit={searchParams.limit as string | undefined}
			left={searchParams.left as string | undefined}
			right={searchParams.right as string | undefined}
		/>
	);
}

function FeedEntriesList(props: { feedId: string; left?: string; right?: string; limit?: string }) {
	const query = createQuery(() =>
		feedEntriesQueryOptions({
			feedId: props.feedId,
			limit: props.limit,
			left: props.left,
			right: props.right,
		})
	);

	const prevHref = () =>
		query.data?.prev_id ? `/feeds/${props.feedId}?left=${query.data.prev_id}` : undefined;

	const nextHref = () =>
		query.data?.next_id ? `/feeds/${props.feedId}?right=${query.data.next_id}` : undefined;

	return (
		<Switch>
			<Match when={query.isError}>
				<FeedEntriesError
					class="mt-8"
					retry={() => {
						query.refetch();
					}}
				/>
			</Match>

			<Match when={query.isLoading}>
				<FeedEntriesSkeleton />
			</Match>

			<Match when={!query.data?.entries.length}>
				<Empty>No entries</Empty>
			</Match>

			<Match when={query.data?.entries.length}>
				<ul class="divide-gray-a3 -mx-3 mb-40 divide-y">
					<For each={query.data?.entries}>
						{(entry) => (
							<Entry.Root entry={entry}>
								<Entry.Content>
									<Entry.Title />
									<Entry.Meta>
										<Entry.Date />
										<Entry.Comments />
										<Entry.ReadToggle />
									</Entry.Meta>
								</Entry.Content>
							</Entry.Root>
						)}
					</For>
				</ul>

				<div class="pwa:bottom-28 fixed right-0 bottom-14 left-0 sm:bottom-0">
					<div class="pointer-events-none mx-auto flex max-w-160 justify-end">
						<Pagination prevHref={prevHref()} nextHref={nextHref()} />
					</div>
				</div>
			</Match>
		</Switch>
	);
}

function FeedEntriesError(props: { class?: string; retry: () => void }) {
	return (
		<div class={"space-y-4" + (props.class ? ` ${props.class}` : "")}>
			<p class="bg-red-a4 p-4">Error loading feed entries</p>

			<Button onClick={props.retry}>Retry</Button>
		</div>
	);
}

function FeedEntriesSkeleton() {
	return (
		<>
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

			<div class="pwa:bottom-28 fixed right-0 bottom-14 left-0 sm:bottom-0">
				<div class="pointer-events-none mx-auto flex max-w-160 justify-end">
					<Pagination prevHref={undefined} nextHref={undefined} />
				</div>
			</div>
		</>
	);
}
