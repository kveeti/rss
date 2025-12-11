import { useParams, useSearchParams } from "@solidjs/router";
import {
	ErrorBoundary,
	For,
	JSX,
	Match,
	Show,
	Suspense,
	Switch,
	createResource,
	splitProps,
} from "solid-js";

import { api } from "../lib/api";
import { API_BASE_URL } from "../lib/constants";

type FeedWithEntryCounts = {
	id: string;
	title: string;
	feed_url: string;
	site_url: string;
	created_at: string;
	entry_count: number;
	unread_entry_count: number;
};

// TODO:
// - loading skeletons
// - error messages
// - pagination button positioning, maybe theres a way to
//   have them not jump around when paginating to
//   between pages with different amounts of entries
// - backend should tell if front should attempt to download favicon
// - move icons under ui/icons/

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
	const [feed] = createResource(props.feedId, async (feedId) => {
		return api<FeedWithEntryCounts>({
			path: `/v1/feeds/${feedId}`,
			method: "GET",
		});
	});

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

	const [entries] = createResource(
		() => ({
			feedId: props.feedId,
			limit: searchParams.limit,
			left: searchParams.left,
			right: searchParams.right,
		}),
		async ({ feedId, limit, left, right }) => {
			const query: Record<string, string> = {};

			if (limit) {
				query.limit = limit as string;
			}
			if (left) {
				query.left = left as string;
			}
			if (right) {
				query.right = right as string;
			}

			return api<{
				entries: Array<{
					id: string;
					title: string;
					url: string;
					comments_url: string;
					published_at: string;
				}>;
				next_id: string;
				prev_id: string;
			}>({
				path: `/v1/feeds/${feedId}/entries`,
				query,
				method: "GET",
			});
		}
	);

	return (
		<div class="relative">
			<ErrorBoundary fallback={<p>Error loading entries</p>}>
				<Suspense fallback={<p>Loading entries...</p>}>
					<div class="sticky top-0 right-0 left-0 z-10 flex justify-end p-2">
						<div class="flex items-center gap-2">
							<Link
								class="bg-gray-1 border-gray-5 focus flex size-8 items-center justify-center rounded-full border"
								href={
									entries()?.prev_id &&
									`/feeds/${props.feedId}?left=${entries()?.prev_id}`
								}
							>
								<IconChevronLeft />
							</Link>

							<Link
								class="bg-gray-1 border-gray-5 focus flex size-8 items-center justify-center rounded-full border"
								href={
									entries()?.next_id &&
									`/feeds/${props.feedId}?right=${entries()?.next_id}`
								}
							>
								<IconChevronRight />
							</Link>
						</div>
					</div>

					<ul class="space-y-2">
						<For each={entries()?.entries}>
							{(entry) => (
								<li class="focus:bg-gray-a2 hover:bg-gray-a2 relative -mx-4 p-4">
									<a href={entry.url} class="focus absolute inset-0">
										<span class="sr-only">{entry.title}</span>
									</a>

									<p class="mb-1">{entry.title}</p>

									<div class="flex items-center gap-2">
										<p class="text-gray-11 text-xs">
											{relativeTime(new Date(entry.published_at))}
										</p>

										<Show when={entry.comments_url}>
											<IconDividerVertical />
											<a
												href={entry.comments_url}
												class="group text-gray-11 relative z-10 -m-4 p-4 text-xs outline-none"
											>
												<span class="in-focus:outline-gray-a10 group-hover:underline in-focus:outline-2 in-focus:outline-offset-2 in-focus:outline-none in-focus:outline-solid">
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
						<div class="flex items-center gap-2">
							<Link
								class="bg-gray-1 border-gray-5 focus flex size-8 items-center justify-center rounded-full border"
								href={
									entries()?.prev_id &&
									`/feeds/${props.feedId}?left=${entries()?.prev_id}`
								}
							>
								<IconChevronLeft />
							</Link>

							<Link
								class="bg-gray-1 border-gray-5 focus flex size-8 items-center justify-center rounded-full border"
								href={
									entries()?.next_id &&
									`/feeds/${props.feedId}?right=${entries()?.next_id}`
								}
							>
								<IconChevronRight />
							</Link>
						</div>
					</div>
				</Suspense>
			</ErrorBoundary>
		</div>
	);
}

const rtf = new Intl.RelativeTimeFormat("en", { numeric: "auto" });
const unitsInSec = [60, 3600, 86400, 86400 * 7, 86400 * 30, 86400 * 365, Infinity];
const unitStrings = ["second", "minute", "hour", "day", "week", "month", "year"];

function relativeTime(date: Date) {
	const secondsDiff = Math.round((date - Date.now()) / 1000);
	const unitIndex = unitsInSec.findIndex((cutoff) => cutoff > Math.abs(secondsDiff));
	const divisor = unitIndex ? unitsInSec[unitIndex - 1] : 1;

	return rtf.format(Math.floor(secondsDiff / divisor), unitStrings[unitIndex]);
}

function Link(allProps: { href?: string | null } & JSX.HTMLAttributes<HTMLAnchorElement>) {
	const [props, rest] = splitProps(allProps, ["href"]);

	return (
		<Switch>
			<Match when={props.href}>
				{(href) => <a role="link" aria-disabled="true" {...rest} href={href()} />}
			</Match>
			<Match when={!props.href}>
				<a role="link" aria-disabled="true" {...rest} />
			</Match>
		</Switch>
	);
}

function IconChevronLeft() {
	return (
		<svg
			width="15"
			height="15"
			viewBox="0 0 15 15"
			fill="none"
			xmlns="http://www.w3.org/2000/svg"
		>
			<path
				d="M8.84182 3.13514C9.04327 3.32401 9.05348 3.64042 8.86462 3.84188L5.43521 7.49991L8.86462 11.1579C9.05348 11.3594 9.04327 11.6758 8.84182 11.8647C8.64036 12.0535 8.32394 12.0433 8.13508 11.8419L4.38508 7.84188C4.20477 7.64955 4.20477 7.35027 4.38508 7.15794L8.13508 3.15794C8.32394 2.95648 8.64036 2.94628 8.84182 3.13514Z"
				fill="currentColor"
				fill-rule="evenodd"
				clip-rule="evenodd"
			></path>
		</svg>
	);
}

function IconChevronRight() {
	return (
		<svg
			width="15"
			height="15"
			viewBox="0 0 15 15"
			fill="none"
			xmlns="http://www.w3.org/2000/svg"
		>
			<path
				d="M6.1584 3.13508C6.35985 2.94621 6.67627 2.95642 6.86514 3.15788L10.6151 7.15788C10.7954 7.3502 10.7954 7.64949 10.6151 7.84182L6.86514 11.8418C6.67627 12.0433 6.35985 12.0535 6.1584 11.8646C5.95694 11.6757 5.94673 11.3593 6.1356 11.1579L9.565 7.49985L6.1356 3.84182C5.94673 3.64036 5.95694 3.32394 6.1584 3.13508Z"
				fill="currentColor"
				fill-rule="evenodd"
				clip-rule="evenodd"
			></path>
		</svg>
	);
}

function IconDividerVertical() {
	return (
		<svg
			width="15"
			height="15"
			viewBox="0 0 15 15"
			fill="none"
			xmlns="http://www.w3.org/2000/svg"
		>
			<path
				d="M7.5 2C7.77614 2 8 2.22386 8 2.5L8 12.5C8 12.7761 7.77614 13 7.5 13C7.22386 13 7 12.7761 7 12.5L7 2.5C7 2.22386 7.22386 2 7.5 2Z"
				fill="currentColor"
				fill-rule="evenodd"
				clip-rule="evenodd"
			></path>
		</svg>
	);
}
