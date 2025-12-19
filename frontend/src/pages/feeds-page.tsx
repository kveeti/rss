import { createAsync, revalidate } from "@solidjs/router";
import {
	ErrorBoundary,
	Suspense,
	createEffect,
	createSignal,
	onCleanup,
	resetErrorBoundaries,
} from "solid-js";

import { Button, buttonStyles } from "../components/button";
import { FeedIcon } from "../components/feed-icon";
import { IconPlus } from "../components/icons/plus";
import { getFeeds } from "./feeds-page.data";

export default function FeedsPage() {
	return (
		<main class="mx-auto max-w-[40rem]">
			<div class="mt-4 mb-8 flex items-center justify-between gap-2">
				<h1 class="font-cool text-3xl font-medium">Feeds</h1>

				<a
					href="/feeds/new"
					class={
						buttonStyles({ variant: "ghost", size: "withIcon" }) +
						" -m-3 inline-flex gap-3"
					}
				>
					<IconPlus class="inline" /> <span>New feed</span>
				</a>
			</div>

			<Feeds />
		</main>
	);
}

function Feeds() {
	return (
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
				<FeedsList />
			</Suspense>
		</ErrorBoundary>
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

function FeedsList() {
	const feeds = createAsync(() => getFeeds());

	return (
		<>
			{!feeds()?.length ? (
				<p class="bg-gray-a2/60 p-4">No feeds yet</p>
			) : (
				<ul class="flex flex-col gap-1">
					{feeds()?.map((feed) => (
						<li class="focus:bg-gray-a2 hover:bg-gray-a2 group/feed relative -mx-4 flex flex-col gap-2 p-4">
							<a
								href={`/feeds/${feed.id}`}
								class="focus absolute top-0 left-0 h-full w-full"
							></a>
							<div class="flex items-center gap-3">
								{feed.has_icon && <FeedIcon feedId={feed.id} class="size-6" />}

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
