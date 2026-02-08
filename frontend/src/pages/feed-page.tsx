import { createAsync, revalidate, useParams, useSearchParams } from "@solidjs/router";
import { ErrorBoundary, For, Show, Suspense } from "solid-js";

import { Button, buttonStyles } from "../components/button";
import { Entry } from "../components/entry";
import { FeedIcon } from "../components/feed-icon";
import { IconSettings } from "../components/icons/settings";
import { Pagination } from "../components/pagination";
import { DefaultNavLinks, Nav, NavWrap, Page } from "../layout";
import { getFeed, getFeedEntries } from "./feed-page.data";

// TODO:
// - pagination button positioning, maybe theres a way to
//   have them not jump around when paginating to
//   between pages with different amounts of entries
// - backend should tell if front should attempt to download favicon

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
					<ErrorBoundary
						fallback={(_error, reset) => (
							<FeedDetailsError
								class="mt-4"
								retry={() => {
									revalidate(getFeed.keyFor(feedId));
									reset();
								}}
							/>
						)}
					>
						<Suspense fallback={<FeedDetailsSkeleton />}>
							<FeedDetails feedId={feedId} />
						</Suspense>
					</ErrorBoundary>

					<FeedEntries feedId={feedId} />
				</main>
			</Page>
		</>
	);
}

function FeedDetails(props: { feedId: string }) {
	const feed = createAsync(() => getFeed(props.feedId));

	return (
		<Show when={feed()} keyed>
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
		</Show>
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
		<ErrorBoundary
			fallback={(_error, reset) => (
				<FeedEntriesError
					class="mt-8"
					retry={() => {
						reset();
						revalidate(
							getFeedEntries.keyFor({
								feedId: props.feedId,
								limit: searchParams.limit as string | undefined,
								left: searchParams.left as string | undefined,
								right: searchParams.right as string | undefined,
							})
						);
					}}
				/>
			)}
		>
			<Suspense fallback={<FeedEntriesSkeleton />}>
				<FeedEntriesList
					feedId={props.feedId}
					limit={searchParams.limit as string | undefined}
					left={searchParams.left as string | undefined}
					right={searchParams.right as string | undefined}
				/>
			</Suspense>
		</ErrorBoundary>
	);
}

function FeedEntriesList(props: { feedId: string; left?: string; right?: string; limit?: string }) {
	const entries = createAsync(() => getFeedEntries(props));

	const prevHref = () =>
		entries()?.prev_id ? `/feeds/${props.feedId}?left=${entries()?.prev_id}` : undefined;

	const nextHref = () =>
		entries()?.next_id ? `/feeds/${props.feedId}?right=${entries()?.next_id}` : undefined;

	return (
		<>
			<ul class="divide-gray-a3 -mx-3 mb-40 divide-y">
				<For each={entries()?.entries}>
					{(entry) => {
						const dateStr = entry.published_at || entry.entry_updated_at;
						const date = dateStr ? new Date(dateStr) : undefined;

						return (
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
						);
					}}
				</For>
			</ul>

			<div class="pwa:bottom-28 fixed right-0 bottom-14 left-0 sm:bottom-0">
				<div class="pointer-events-none mx-auto flex max-w-160 justify-end">
					<Pagination prevHref={prevHref()} nextHref={nextHref()} />
				</div>
			</div>
		</>
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

const rtf = new Intl.RelativeTimeFormat("en", { numeric: "auto" });
const unitsInSec = [60, 3600, 86400, 86400 * 7, 86400 * 30, 86400 * 365, Infinity];
const unitStrings = ["second", "minute", "hour", "day", "week", "month", "year"] as const;

function relativeTime(date: Date) {
	const secondsDiff = Math.round((date.getTime() - Date.now()) / 1000);
	const unitIndex = unitsInSec.findIndex((cutoff) => cutoff > Math.abs(secondsDiff));
	const divisor = unitIndex ? unitsInSec[unitIndex - 1] : 1;

	return rtf.format(Math.floor(secondsDiff / divisor), unitStrings[unitIndex]);
}
