import { createAsync, revalidate } from "@solidjs/router";
import {
	ErrorBoundary,
	Suspense,
	createEffect,
	createSignal,
	onCleanup,
	resetErrorBoundaries,
} from "solid-js";

import { FeedIcon } from "../components/feed-icon";
import { Button, buttonStyles } from "../ui/button";
import { IconPlus } from "../ui/icons/plus";
import { Feed, getFeeds } from "./feeds-page.data";

export default function FeedsPage() {
	const feeds = createAsync(() => getFeeds());

	return (
		<main class="mx-auto max-w-[40rem]">
			<div class="mt-4 mb-8 flex items-center justify-between gap-2">
				<h1 class="text-3xl font-bold">Feeds</h1>

				<a
					href="/feeds/new"
					class={buttonStyles({ variant: "ghost" }) + " -m-4 inline-flex gap-3"}
				>
					<IconPlus class="inline" /> <span>New feed</span>
				</a>
			</div>

			<ErrorBoundary
				fallback={
					<FeedsListError
						retry={() => {
							revalidate(getFeeds.key);
							resetErrorBoundaries();
						}}
					/>
				}
			>
				<Suspense fallback={<FeedsListSkeleton />}>
					<FeedsList feeds={feeds()} />
				</Suspense>
			</ErrorBoundary>
		</main>
	);
}

function DelayedLoadingAnnouncement(props: {
	message: string;
	isLoading: boolean;
	delayMS?: number;
}) {
	const [showMessage, setShowMessage] = createSignal(false);
	let timeout: number | null = null;

	createEffect(() => {
		if (props.isLoading) {
			timeout = setTimeout(() => {
				setShowMessage(true);
			}, props.delayMS ?? 200);
		}
	});

	onCleanup(() => {
		if (timeout) {
			clearTimeout(timeout);
		}
		setShowMessage(false);
	});

	return (
		<div role="status" aria-live="polite" aria-atomic="true" class="sr-only">
			{showMessage() && props.message}
		</div>
	);
}

function FeedsListError(props: { retry: () => void }) {
	return (
		<div class="space-y-4">
			<p class="bg-red-a4 p-4">Error loading feeds</p>

			<Button onClick={props.retry}>Retry</Button>
		</div>
	);
}

function FeedsListSkeleton() {
	return (
		<>
			<DelayedLoadingAnnouncement isLoading message="Loading feeds" />

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
		</>
	);
}

function FeedsList(props: { feeds?: Array<Feed> }) {
	return (
		<>
			{!props.feeds?.length ? (
				<p class="bg-gray-a2/60 p-4">No feeds yet</p>
			) : (
				<ul class="flex flex-col gap-1">
					{props.feeds?.map((feed) => (
						<li class="focus:bg-gray-a2 hover:bg-gray-a2 group/feed relative -mx-4 flex flex-col gap-2 p-4">
							<a
								href={`/feeds/${feed.id}`}
								class="focus absolute top-0 left-0 h-full w-full"
							></a>
							<div class="flex items-center gap-3">
								<FeedIcon feedId={feed.id} class="size-6" />

								<div class="flex items-center gap-2 font-medium">
									<span class="font-cool inline text-[1.3rem] group-hover/feed:underline group-has-[a[id=site]:hover]/feed:no-underline">
										{feed.title}
									</span>
									<a
										id="site"
										href={feed.site_url}
										class="group/link text-gray-11 relative z-10 -m-4 p-4 text-xs outline-none"
									>
										<span class="in-focus:outline-gray-a10 underline group-hover/link:text-white in-focus:outline-2 in-focus:outline-offset-2 in-focus:outline-none in-focus:outline-solid">
											{feed.site_url
												.replace(/^https?:\/\//, "")
												.replace(/\/$/, "")}
										</span>
									</a>
								</div>
							</div>

							<p class="text-gray-11 text-sm">
								{feed.entry_count} entries ({feed.unread_entry_count} unread)
							</p>
						</li>
					))}
				</ul>
			)}
		</>
	);
}
