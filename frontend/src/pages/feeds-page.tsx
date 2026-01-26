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
import { Empty } from "../components/empty";
import { FeedIcon } from "../components/feed-icon";
import { IconPlus } from "../components/icons/plus";
import { DefaultNavLinks, Nav, NavWrap, Page } from "../layout";
import { prettifyUrl } from "../lib/urls";
import { getFeeds } from "./feeds-page.data";

export default function FeedsPage() {
	return (
		<>
			<NavWrap>
				<Nav>
					<DefaultNavLinks />
				</Nav>
			</NavWrap>

			<Page>
				<main class="mx-auto max-w-[40rem] px-3">
					<div class="mb-4 flex items-center justify-between gap-2">
						<h1 class="font-cool text-3xl font-medium">Feeds</h1>

						<a
							href="/feeds/new"
							class={
								buttonStyles({ variant: "ghost", size: "withIcon" }) +
								" inline-flex gap-3"
							}
						>
							<IconPlus class="inline" /> <span>New feed</span>
						</a>
					</div>

					<Feeds />
				</main>
			</Page>
		</>
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
				<Empty>No feeds yet</Empty>
			) : (
				<ul class="divide-gray-a3 -mx-3 mb-40 divide-y">
					{feeds()?.map((feed) => (
						<li class="focus:bg-gray-a2 hover:bg-gray-a2 group/feed relative flex flex-col gap-2 p-3">
							<a
								href={`/feeds/${feed.id}`}
								class="focus absolute top-0 left-0 h-full w-full"
							></a>
							<div class="flex gap-3">
								{feed.has_icon && (
									<div class="font-cool flex h-[1lh] flex-shrink-0 items-center justify-center text-[1.3rem]">
										<FeedIcon feedId={feed.id} class="size-6" />
									</div>
								)}

								<div class="flex flex-col gap-3 font-medium">
									<div class="flex flex-col gap-1">
										<span class="font-cool inline text-[1.3rem] group-hover/feed:underline group-has-[a[id=site]:hover]/feed:no-underline">
											{feed.title}
										</span>

										<a
											id="site"
											href={feed.site_url}
											class="group/link text-gray-11 relative z-10 -m-4 max-w-max p-4 text-xs outline-none"
										>
											<span class="in-focus:outline-gray-a10 underline group-hover/link:text-white in-focus:outline-2 in-focus:outline-offset-2 in-focus:outline-none in-focus:outline-solid">
												{prettifyUrl(feed.site_url)}
											</span>
										</a>
									</div>

									<p class="text-gray-11 text-sm">
										{feed.entry_count} entries ({feed.unread_entry_count}{" "}
										unread)
									</p>
								</div>
							</div>
						</li>
					))}
				</ul>
			)}
		</>
	);
}
