import { createAsync, revalidate, useParams } from "@solidjs/router";
import {
	ErrorBoundary,
	Match,
	Suspense,
	Switch,
	createSignal,
	resetErrorBoundaries,
} from "solid-js";

import { Button } from "../components/button";
import { FeedIcon } from "../components/feed-icon";
import { IconCheck } from "../components/icons/check";
import { IconUpdate } from "../components/icons/update";
import { Input } from "../components/input";
import { DefaultNavLinks, Nav, NavWrap, Page } from "../layout";
import { api } from "../lib/api";
import { FeedWithEntryCounts, getFeed, getFeedEntries } from "./feed-page.data";

export default function FeedEditPage() {
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
				<main class="mx-auto max-w-90 px-3">
					<ErrorBoundary
						fallback={(_error, reset) => (
							<Err
								class="mt-8"
								retry={() => {
									revalidate(getFeed.keyFor(feedId));
									reset();
									resetErrorBoundaries();
								}}
							/>
						)}
					>
						<Suspense fallback={<Skeleton />}>
							<FeedEdit feedId={feedId} />
						</Suspense>
					</ErrorBoundary>
				</main>
			</Page>
		</>
	);
}

function Err(props: { class?: string; retry: () => void }) {
	return (
		<div class={"mx-auto max-w-80 space-y-4" + (props.class ? ` ${props.class}` : "")}>
			<p class="bg-red-a4 p-4">Error loading feed details</p>

			<Button onClick={props.retry}>Retry</Button>
		</div>
	);
}

function Skeleton() {
	return (
		<>
			<div class="mx-auto flex max-w-97.5 items-center gap-3">
				<div class="bg-gray-a2/40 size-5.5" />

				<h1 class="bg-gray-a2/40 font-cool w-[40%] text-xl">
					<span class="invisible">0</span>
				</h1>
			</div>

			<div class="mx-auto mt-4 max-w-80">
				<div class="bg-gray-a2/40 max-w-[90%]">
					<span class="invisible text-sm">0</span>
				</div>

				<div class="mx-auto mt-8 max-w-80 space-y-8">
					<div class="flex items-center justify-between gap-4">
						<div class="w-full space-y-1">
							<div class="bg-gray-a2/40">
								<p class="invisible text-sm">0</p>
							</div>
							<div class="bg-gray-a2/40 w-[70%]">
								<p class="invisible text-sm">0</p>
							</div>
						</div>

						<div class="w-full max-w-max">
							<div class="bg-gray-a2/40 size-10"></div>
						</div>
					</div>

					<div class="space-y-6">
						<div class="bg-gray-a2/40 h-10 w-full"></div>

						<div class="bg-gray-a2/40 h-10 w-full"></div>

						<div class="bg-gray-a2/40 h-10 w-full"></div>

						<div class="flex justify-end">
							<div class="bg-gray-a2/40 h-10 px-4">
								<span class="invisible">Save</span>
							</div>
						</div>
					</div>
				</div>
			</div>
		</>
	);
}

function FeedEdit(props: { feedId: string }) {
	const queriedFeed = createAsync(() => getFeed(props.feedId));
	const [latestFeed, setLatestFeed] = createSignal<FeedWithEntryCounts | null>(null);

	const feed = () => latestFeed() ?? queriedFeed();

	return (
		<>
			{feed() ? (
				<>
					<div class="font-cool relative mx-auto -my-2 -ms-12 py-2 ps-12 text-xl">
						{feed()!.has_icon && (
							<FeedIcon
								feedId={feed()!.id}
								class="me-2.5 inline size-5.5 align-text-bottom min-[27rem]:-ms-8.5 min-[27rem]:me-3"
							/>
						)}
						<h1 class="inline font-medium">{feed()!.title}</h1>

						<a href={feed()!.site_url} class="absolute inset-0">
							<span class="sr-only">{feed()!.title}</span>
						</a>
					</div>

					<div class="mx-auto mt-4">
						<p class="text-gray-11 text-sm">
							{feed()!.entry_count} entries ({feed()!.unread_entry_count} unread)
						</p>

						<div class="mx-auto mt-8 space-y-8">
							<div class="flex items-center justify-between gap-4">
								<p>
									<span class="text-gray-a11">Last synced at:</span>
									<br />
									{feed()!.last_synced_at ? (
										<span>
											{Intl.DateTimeFormat(undefined, {
												year: "numeric",
												month: "numeric",
												day: "numeric",
												hour: "numeric",
												minute: "numeric",
												second: "numeric",
												hour12: false,
											}).format(new Date(feed()!.last_synced_at!))}
										</span>
									) : (
										<span>never</span>
									)}
								</p>

								<SyncButton feedId={feed()!.id} setLatestFeed={setLatestFeed} />
							</div>
							<form class="space-y-6">
								<Input label="Title" value={feed()!.title} />

								<Input label="Site URL" value={feed()!.site_url} />

								<Input label="Feed URL" value={feed()!.feed_url} />

								<div class="flex justify-end">
									<Button>Save</Button>
								</div>
							</form>

							<hr class="border-gray-a3 border-t" />

							<div class="space-x-2">
								<Button variant="destructive">Delete</Button>
							</div>
						</div>
					</div>
				</>
			) : null}
		</>
	);
}

function SyncButton(props: {
	feedId: string;
	setLatestFeed: (latestFeed: FeedWithEntryCounts) => void;
}) {
	const [syncStatus, setSyncStatus] = createSignal<"idle" | "syncing" | "synced" | "error">(
		"idle"
	);

	async function onSyncClick() {
		setSyncStatus("syncing");

		const latestFeed = await api<FeedWithEntryCounts>({
			method: "POST",
			path: `/v1/feeds/${props.feedId}/sync`,
		});

		setSyncStatus("synced");
		props.setLatestFeed(latestFeed);
		revalidate(getFeedEntries.key);
		revalidate(getFeed.keyFor(props.feedId));

		setTimeout(() => {
			setSyncStatus("idle");
		}, 2000);
	}

	return (
		<Button onclick={onSyncClick} size="icon" variant="ghost" class="ms-2" title="Sync now">
			<Switch>
				<Match when={syncStatus() === "idle"}>
					<IconUpdate />
				</Match>

				<Match when={syncStatus() === "syncing"}>
					<IconUpdate class="animate-spin" />
				</Match>

				<Match when={syncStatus() === "synced"}>
					<IconCheck />
				</Match>
			</Switch>
		</Button>
	);
}
