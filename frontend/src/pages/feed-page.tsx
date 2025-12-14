import { createAsync, useParams, useSearchParams } from "@solidjs/router";
import { ErrorBoundary, For, JSX, Match, Show, Suspense, Switch, splitProps } from "solid-js";

import { API_BASE_URL } from "../lib/constants";
import { IconChevronLeft } from "../ui/icons/chevron-left";
import { IconChevronRight } from "../ui/icons/chevron-right";
import { IconDividerVertical } from "../ui/icons/divider-vertical";
import { getFeed, getFeedEntries } from "./feed-page.data";

// TODO:
// - loading skeletons
// - error messages
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
		<main class="mx-auto max-w-[40rem]">
			<FeedDetails feedId={feedId} />
			<FeedEntries feedId={feedId} />
		</main>
	);
}

function FeedError() {
	return <p>Error loading feed</p>;
}

function FeedSkeleton() {
	return <p>Loading feed...</p>;
}

function FeedDetails(props: { feedId: string }) {
	const feed = createAsync(() => getFeed(props.feedId));

	return (
		<ErrorBoundary fallback={<FeedError />}>
			<Suspense fallback={<FeedSkeleton />}>
				<Show when={feed()}>
					{(feed) => (
						<div>
							<div class="mt-4 mb-4 flex items-center justify-between gap-4">
								<div class="relative flex items-center gap-4">
									<a href={feed().site_url} class="absolute inset-0">
										<span class="sr-only">{feed().title}</span>
									</a>

									<img
										class="size-6"
										src={API_BASE_URL + `/v1/feeds/${feed().id}/icon`}
										aria-hidden="true"
									/>

									<h1 class="text-2xl font-bold">{feed().title}</h1>
								</div>
							</div>
						</div>
					)}
				</Show>
			</Suspense>
		</ErrorBoundary>
	);
}

function FeedEntries(props: { feedId: string }) {
	const [searchParams] = useSearchParams();

	const entries = createAsync(() =>
		getFeedEntries({
			feedId: props.feedId,
			limit: searchParams.limit as string | undefined,
			left: searchParams.left as string | undefined,
			right: searchParams.right as string | undefined,
		})
	);

	return (
		<div class="relative">
			<ErrorBoundary fallback={<p>Error loading entries</p>}>
				<Suspense fallback={<p>Loading entries...</p>}>
					<div class="sticky top-0 right-0 left-0 z-10 flex justify-end p-2">
						<Pagination
							nextId={entries()?.next_id}
							prevId={entries()?.prev_id}
							feedId={props.feedId}
						/>
					</div>

					<ul class="space-y-2">
						<For each={entries()?.entries}>
							{(entry) => (
								<li class="group/entry focus:bg-gray-a2 hover:bg-gray-a2 relative -mx-4 p-4">
									<a
										href={entry.url}
										target="_blank"
										class="focus absolute inset-0"
									>
										<span class="sr-only">{entry.title}</span>
									</a>

									<p class="font-cool mb-1 text-[1.3rem] font-[200] group-hover/entry:underline group-has-[a[id=comments]:hover]/entry:no-underline">
										{entry.title}
									</p>

									<div class="flex items-center gap-2">
										<p class="text-gray-11 text-sm">
											{relativeTime(new Date(entry.published_at))}
										</p>

										<Show when={entry.comments_url}>
											<IconDividerVertical />
											<a
												id="comments"
												href={entry.comments_url}
												target="_blank"
												class="group/comments text-gray-11 relative z-10 -m-4 p-4 text-sm underline outline-none"
											>
												<span class="in-focus:outline-gray-a10 group-hover/comments:text-white in-focus:outline-2 in-focus:outline-offset-2 in-focus:outline-none in-focus:outline-solid">
													comments
												</span>
											</a>
										</Show>
									</div>
								</li>
							)}
						</For>
					</ul>

					<div class="sticky right-0 bottom-0 left-0 flex justify-end p-2">
						<Pagination
							nextId={entries()?.next_id}
							prevId={entries()?.prev_id}
							feedId={props.feedId}
						/>
					</div>
				</Suspense>
			</ErrorBoundary>
		</div>
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

function Link(allProps: { href?: string | null } & JSX.HTMLAttributes<HTMLAnchorElement>) {
	const [props, rest] = splitProps(allProps, ["href"]);

	return (
		<Switch>
			<Match when={props.href}>{(href) => <a role="link" {...rest} href={href()} />}</Match>
			<Match when={!props.href}>
				<a role="link" aria-disabled="true" {...rest} />
			</Match>
		</Switch>
	);
}

function Pagination(props: { nextId?: string; prevId?: string; feedId: string }) {
	return (
		<div class="flex items-center gap-2">
			<Link
				class="bg-gray-1 border-gray-5 focus flex size-8 items-center justify-center rounded-full border aria-disabled:opacity-40"
				href={props.prevId && `/feeds/${props.feedId}?left=${props.prevId}`}
			>
				<IconChevronLeft />
			</Link>

			<Link
				class="bg-gray-1 border-gray-5 focus flex size-8 items-center justify-center rounded-full border aria-disabled:opacity-40"
				href={props.nextId && `/feeds/${props.feedId}?right=${props.nextId}`}
			>
				<IconChevronRight />
			</Link>
		</div>
	);
}
