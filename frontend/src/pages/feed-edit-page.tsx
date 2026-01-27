import { createAsync, revalidate, useNavigate, useParams } from "@solidjs/router";
import {
	ErrorBoundary,
	Match,
	Show,
	Suspense,
	Switch,
	createSignal,
	resetErrorBoundaries,
} from "solid-js";
import { createStore } from "solid-js/store";

import { Button } from "../components/button";
import { FeedIcon } from "../components/feed-icon";
import { IconCheck } from "../components/icons/check";
import { IconCross } from "../components/icons/cross";
import { IconUpdate } from "../components/icons/update";
import { Input } from "../components/input";
import { DefaultNavLinks, Nav, NavWrap, Page } from "../layout";
import { api } from "../lib/api";
import { FeedWithEntryCounts, getFeed, getFeedEntries } from "./feed-page.data";
import { getFeeds } from "./feeds-page.data";

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
	const syncError = () => formatSyncError(feed()?.last_sync_result ?? null);

	return (
		<>
			{feed() ? (
				<>
					<div class="font-cool relative mx-auto -my-2 -ms-12 py-2 ps-12 text-xl">
						<FeedIcon
							class="me-2.5 inline size-5.5 align-text-bottom min-[27rem]:-ms-8.5 min-[27rem]:me-3"
							feedId={feed()!.id}
							hasIcon={feed()!.has_icon}
							fallbackUrl={feed()!.feed_url}
						/>
						<h1 class="inline font-medium">{feed()?.title}</h1>

						<a href={feed()?.site_url ?? feed()?.feed_url} class="absolute inset-0">
							<span class="sr-only">{feed()?.title}</span>
						</a>
					</div>

					<div class="mx-auto mt-4">
						<p class="text-gray-11 text-sm">
							{feed()?.entry_count} entries ({feed()?.unread_entry_count} unread)
						</p>

						<div class="mx-auto mt-8 space-y-8">
							<div class="flex items-center justify-between gap-4">
								<p>
									<span class="text-gray-a11">Last synced at:</span>
									<br />
									{feed()?.last_synced_at ? (
										<span>
											{Intl.DateTimeFormat(undefined, {
												year: "numeric",
												month: "numeric",
												day: "numeric",
												hour: "numeric",
												minute: "numeric",
												second: "numeric",
												hour12: false,
											}).format(new Date(feed()?.last_synced_at!))}
										</span>
									) : (
										<span>never</span>
									)}
								</p>

								<SyncButton feedId={feed()!.id} setLatestFeed={setLatestFeed} />
							</div>
							{syncError() ? (
								<p class="bg-red-a4 border-red-a6 border p-4 text-sm">
									{syncError()}
								</p>
							) : null}
							<EditForm
								feed={feed()!}
								feedId={props.feedId}
								onUpdated={(updatedFeed) => setLatestFeed(updatedFeed)}
							/>

							<hr class="border-gray-a3 border-t" />

							<DeleteSection feedId={props.feedId} />
						</div>
					</div>
				</>
			) : null}
		</>
	);
}

function EditForm(props: {
	feed: FeedWithEntryCounts;
	feedId: string;
	onUpdated: (feed: FeedWithEntryCounts) => void;
}) {
	const [state, setState] = createStore({
		saveStatus: "idle" as "idle" | "saving" | "saved" | "error",
		saveError: null as string | null,
	});
	const isSaving = () => state.saveStatus === "saving";

	async function onSubmit(event: SubmitEvent) {
		event.preventDefault();

		setState({
			saveStatus: "saving",
			saveError: null,
		});

		try {
			const form = event.currentTarget as HTMLFormElement;
			const data = new FormData(form);
			const title = String(data.get("title") ?? "").trim();
			const siteUrl = String(data.get("siteUrl") ?? "").trim();
			const feedUrl = String(data.get("feedUrl") ?? "").trim();

			const updatedFeed = await api<FeedWithEntryCounts>({
				method: "PUT",
				path: `/v1/feeds/${props.feedId}`,
				body: {
					title,
					feed_url: feedUrl,
					site_url: siteUrl ? siteUrl : null,
				},
			});

			props.onUpdated(updatedFeed);
			setState("saveStatus", "saved");
			revalidate(getFeed.keyFor(props.feedId));
			revalidate(getFeeds.key);

			setTimeout(() => {
				setState((prev) => {
					if (prev.saveStatus === "saved") {
						return { ...prev, saveStatus: "idle" };
					}
					return prev;
				});
			}, 2000);
		} catch (error) {
			setState({
				saveStatus: "error",
				saveError: error instanceof Error ? error.message : "Error saving feed",
			});
		}
	}

	return (
		<form class="space-y-6" onSubmit={onSubmit}>
			<Input label="Title" name="title" value={props.feed.title} required />

			<Input label="Site URL" name="siteUrl" value={props.feed.site_url ?? ""} />

			<Input label="Feed URL" name="feedUrl" value={props.feed.feed_url} required />

			<div class="flex justify-end">
				<div class="flex items-center gap-3">
					{state.saveError ? (
						<p class="bg-red-a4 border-red-a6 border p-3 text-sm">{state.saveError}</p>
					) : state.saveStatus === "saved" ? (
						<p class="text-green-11 text-sm">Saved.</p>
					) : null}
					<Button type="submit" isLoading={isSaving()}>
						Save
					</Button>
				</div>
			</div>
		</form>
	);
}

function DeleteSection(props: { feedId: string }) {
	const navigate = useNavigate();
	const [state, setState] = createStore({
		meta: {
			deleteError: null as string | null,
			isDeleting: false,
		},
	});

	async function onDelete() {
		if (state.meta.isDeleting) return;
		if (!confirm("Delete this feed? This will remove all entries.")) return;

		setState("meta", {
			isDeleting: true,
			deleteError: null,
		});

		try {
			await api<void>({
				method: "DELETE",
				path: `/v1/feeds/${props.feedId}`,
			});
			revalidate(getFeeds.key);
			navigate("/feeds");
		} catch (error) {
			setState("meta", {
				deleteError: error instanceof Error ? error.message : "Failed to delete feed",
			});
		} finally {
			setState("meta", {
				isDeleting: false,
			});
		}
	}

	return (
		<div class="space-y-3">
			<div class="space-x-2">
				<Button variant="destructive" onClick={onDelete} isLoading={state.meta.isDeleting}>
					Delete
				</Button>
			</div>
			{state.meta.deleteError ? (
				<p class="bg-red-a4 border-red-a6 border p-3 text-sm">{state.meta.deleteError}</p>
			) : null}
		</div>
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

		try {
			const latestFeed = await api<FeedWithEntryCounts>({
				method: "POST",
				path: `/v1/feeds/${props.feedId}/sync`,
			});

			setSyncStatus(latestFeed.last_sync_result === "success" ? "synced" : "error");
			props.setLatestFeed(latestFeed);
			revalidate(getFeedEntries.key);
			revalidate(getFeed.keyFor(props.feedId));
		} catch (_error) {
			setSyncStatus("error");
		} finally {
			setTimeout(() => {
				setSyncStatus("idle");
			}, 2000);
		}
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
				<Match when={syncStatus() === "error"}>
					<IconCross />
				</Match>
			</Switch>
		</Button>
	);
}

function formatSyncError(result: string | null) {
	if (!result || result === "success") {
		return null;
	}

	switch (result) {
		case "parse_error":
			return "Last sync failed: feed parse error";
		case "not_found":
			return "Last sync failed: feed not found";
		case "disallowed":
			return "Last sync failed: disallowed by robots.txt";
		case "needs_choice":
			return "Last sync failed: multiple feeds found";
		case "unexpected_html":
			return "Last sync failed: expected feed but got html";
		case "invalid_url":
			return "Last sync failed: invalid url";
		case "fetch_error":
			return "Last sync failed: network or fetch error";
		case "db_error":
			return "Last sync failed: database error";
		default:
			return "Last sync failed: unexpected error";
	}
}
