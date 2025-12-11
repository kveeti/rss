import { ErrorBoundary, For, Suspense, createResource, resetErrorBoundaries } from "solid-js";

import { api } from "../lib/api";
import { API_BASE_URL } from "../lib/constants";
import { Button } from "../ui/button";

export default function FeedsPage() {
	return (
		<main class="mx-auto max-w-[40rem]">
			<h1 class="mt-4 mb-8 text-3xl font-bold">Feeds</h1>

			<ErrorBoundary fallback={<FeedsListError />}>
				<Suspense fallback={<FeedsListSkeleton />}>
					<FeedsList />
				</Suspense>
			</ErrorBoundary>
		</main>
	);
}

const [feeds, { refetch }] = createResource(() => {
	return api<
		Array<{
			id: string;
			title: string;
			feed_url: string;
			site_url: string;
			created_at: string;
			entry_count: number;
			unread_entry_count: number;
		}>
	>({
		path: "/v1/feeds",
		method: "GET",
	});
});

function FeedsListError() {
	return (
		<div class="space-y-4">
			<p class="bg-red-a4 p-4">Error loading feeds</p>

			<Button
				onClick={() => {
					refetch();
					resetErrorBoundaries();
				}}
			>
				Retry
			</Button>
		</div>
	);
}

function FeedsListSkeleton() {
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

function FeedsList() {
	return (
		<ul class="flex flex-col gap-1">
			<For each={feeds()}>
				{(feed) => (
					<li class="focus:bg-gray-a2 hover:bg-gray-a2 relative -mx-4 flex flex-col gap-2 p-4">
						<a
							href={`/feeds/${feed.id}`}
							class="focus absolute top-0 left-0 h-full w-full"
						></a>
						<div class="flex items-center gap-3">
							<img
								class="inline-flex size-6"
								src={API_BASE_URL + `/v1/feeds/${feed.id}/icon`}
							/>

							<div class="flex items-center gap-2 font-medium">
								<span class="inline">{feed.title}</span>{" "}
								<a
									href={feed.site_url}
									class="group text-gray-11 relative z-10 -m-4 p-4 text-xs outline-none"
								>
									<span class="in-focus:outline-gray-a10 group-hover:underline in-focus:outline-2 in-focus:outline-offset-2 in-focus:outline-none in-focus:outline-solid">
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
				)}
			</For>
		</ul>
	);
}
