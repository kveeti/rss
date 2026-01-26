import { createAsync, revalidate, useSearchParams } from "@solidjs/router";
import { ErrorBoundary, JSX, Show, Suspense, createSignal, resetErrorBoundaries } from "solid-js";

import { Button } from "../components/button";
import { Empty } from "../components/empty";
import { Entry } from "../components/entry";
import { NavPaginationLinks, Pagination, buildPaginatedHref } from "../components/pagination";
import { Select } from "../components/select";
import { DefaultNavLinks, Nav, NavWrap } from "../layout";
import { type FilterParams, queryEntries } from "./entries-page.data";

export default function EntriesPage() {
	const [searchParams] = useSearchParams();

	const filterParams = (): FilterParams => ({
		feed_id: searchParams.feed_id as string | undefined,
		query: searchParams.query as string | undefined,
		left: searchParams.left as string | undefined,
		right: searchParams.right as string | undefined,
		unread: searchParams.unread as string | undefined,
		starred: searchParams.starred as string | undefined,
		start: searchParams.start as string | undefined,
		end: searchParams.end as string | undefined,
		sort: searchParams.sort as string | undefined,
	});

	return (
		<>
			<NavWrap>
				<Nav>
					<div class="flex w-full justify-between">
						<DefaultNavLinks />
						<NavPagination {...filterParams()} />
					</div>
				</Nav>
			</NavWrap>

			<main class="mx-auto mt-14 max-w-160 px-3">
				<FilterBar />

				<ErrorBoundary
					fallback={(_error, _reset) => (
						<EntriesListError
							class="mt-4"
							retry={() => {
								revalidate(queryEntries.keyFor(filterParams()));
								// Reset all error boundaries here so that
								// the one in nav also get reset
								resetErrorBoundaries();
							}}
						/>
					)}
				>
					<Suspense fallback={<Pagination />}>
						<EntriesList {...filterParams()} />
					</Suspense>
				</ErrorBoundary>
			</main>
		</>
	);
}

function EntriesListError(props: { class?: string; retry: () => void }) {
	return (
		<div class={"space-y-4" + (props.class ? ` ${props.class}` : "")}>
			<p class="bg-red-a4 p-4">Error loading feed details</p>

			<Button onClick={props.retry}>Retry</Button>
		</div>
	);
}

function NavPagination(props: FilterParams) {
	return (
		<ErrorBoundary fallback={<NavPaginationLinks />}>
			<Suspense fallback={<NavPaginationLinks />}>
				<NavPaginationInner {...props} />
			</Suspense>
		</ErrorBoundary>
	);
}

function NavPaginationInner(props: FilterParams) {
	const entriesCursor = createAsync(() => queryEntries(props));
	const [searchParams] = useSearchParams();

	const nextHref = () =>
		buildPaginatedHref("right", entriesCursor()?.next_id, "/entries", searchParams);
	const prevHref = () =>
		buildPaginatedHref("left", entriesCursor()?.prev_id, "/entries", searchParams);

	return <NavPaginationLinks nextHref={nextHref()} prevHref={prevHref()} />;
}

function FilterBar() {
	const [searchParams, setSearchParams] = useSearchParams();
	const [searchValue, setSearchValue] = createSignal((searchParams.query as string) || "");

	const updateFilters = (updates: Record<string, string | undefined>) => {
		const newParams: Record<string, string | undefined> = {
			...updates,
			// Reset pagination when filters change
			left: undefined,
			right: undefined,
		};

		setSearchParams(newParams);
	};

	const handleSearch = (e: Event) => {
		e.preventDefault();
		const value = searchValue().trim();
		updateFilters({ query: value || undefined });
	};

	const toggleUnread = () => {
		updateFilters({
			unread: searchParams.unread === "true" ? undefined : "true",
		});
	};

	const toggleStarred = () => {
		updateFilters({
			starred: searchParams.starred === "true" ? undefined : "true",
		});
	};

	const setTimePreset = (preset: string) => {
		const now = new Date();
		let start: string | undefined;
		let end: string | undefined;

		switch (preset) {
			case "today": {
				const todayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate());
				start = todayStart.toISOString();
				break;
			}
			case "week": {
				const weekAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000);
				start = weekAgo.toISOString();
				break;
			}
			case "month": {
				const monthAgo = new Date(now.getTime() - 30 * 24 * 60 * 60 * 1000);
				start = monthAgo.toISOString();
				break;
			}
			default:
				start = undefined;
				end = undefined;
		}

		updateFilters({ start, end });
	};

	const setSort = (sort: string) => {
		updateFilters({ sort: sort === "newest" ? undefined : sort });
	};

	const clearFilters = () => {
		setSearchValue("");
		const toClear = { ...searchParams };
		Object.keys(toClear).forEach((key) => {
			toClear[key] = undefined;
		});
		setSearchParams(toClear);
	};

	const hasActiveFilters = () => {
		return (
			searchParams.query ||
			searchParams.unread ||
			searchParams.starred ||
			searchParams.start ||
			searchParams.end ||
			(searchParams.sort && searchParams.sort !== "newest")
		);
	};

	const currentTimePreset = () => {
		if (!searchParams.start) return "";
		// Simple detection - just check if there's a start date
		return "custom";
	};

	return (
		<div class="border-gray-a3 mb-4 flex flex-wrap items-center gap-2 border-b pb-4">
			<form onSubmit={handleSearch} class="flex items-center gap-2">
				<input
					type="search"
					placeholder="Search..."
					value={searchValue()}
					onInput={(e) => setSearchValue(e.currentTarget.value)}
					class="focus border-gray-a6 h-8 w-40 border px-2 text-sm"
				/>
				<button
					type="submit"
					class="focus border-gray-a5 bg-gray-4 h-8 border px-2 text-sm"
				>
					Search
				</button>
			</form>

			<Select
				value={currentTimePreset()}
				onChange={(e) => setTimePreset(e.currentTarget.value)}
				class="focus border-gray-a6 h-8 border px-2 text-sm"
			>
				<option value="">All time</option>
				<option value="today">Today</option>
				<option value="week">Last 7 days</option>
				<option value="month">Last 30 days</option>
			</Select>

			<Select
				value={(searchParams.sort as string) || "newest"}
				onChange={(e) => setSort(e.currentTarget.value)}
				class="focus border-gray-a6 h-8 border px-2 text-sm"
			>
				<option value="newest">Newest</option>
				<option value="oldest">Oldest</option>
			</Select>

			<div class="flex flex-wrap gap-2">
				<FilterChip active={searchParams.unread === "true"} onClick={toggleUnread}>
					Unread
				</FilterChip>

				<FilterChip active={searchParams.starred === "true"} onClick={toggleStarred}>
					Starred
				</FilterChip>

				<Show when={hasActiveFilters()}>
					<button
						onClick={clearFilters}
						class="focus text-gray-11 h-8 px-2 text-sm hover:underline"
					>
						Clear
					</button>
				</Show>
			</div>
		</div>
	);
}

function FilterChip(props: { active: boolean; onClick: () => void; children: JSX.Element }) {
	return (
		<button
			onClick={props.onClick}
			class={`focus h-8 border px-3 text-sm ${
				props.active ? "bg-gray-4 border-gray-a5" : "border-gray-a3 hover:border-gray-a5"
			}`}
		>
			{props.children}
		</button>
	);
}

function EntriesList(props: FilterParams) {
	const entriesCursor = createAsync(() => queryEntries(props));
	const [searchParams] = useSearchParams();

	const nextHref = () =>
		buildPaginatedHref("right", entriesCursor()?.next_id, "/entries", searchParams);
	const prevHref = () =>
		buildPaginatedHref("left", entriesCursor()?.prev_id, "/entries", searchParams);

	return (
		<>
			{!entriesCursor()?.entries.length ? (
				<Empty>No matches</Empty>
			) : (
				<div>
					<ul class="divide-gray-a3 -mx-3 mb-40 divide-y">
						{entriesCursor()?.entries?.map((entry) => {
							const dateStr = entry.published_at || entry.entry_updated_at;
							const date = dateStr ? new Date(dateStr) : undefined;

							return (
								<Entry
									feedId={entry.feed_id}
									hasIcon={entry.has_icon}
									title={entry.title}
									date={date}
									commentsUrl={entry.comments_url}
									url={entry.url}
								/>
							);
						})}
					</ul>

					<div class="pwa:bottom-28 pointer-events-none fixed right-0 bottom-13 left-0 -me-6 sm:bottom-0">
						<div class="mx-auto flex max-w-160 justify-end">
							<Pagination prevHref={prevHref()} nextHref={nextHref()} />
						</div>
					</div>
				</div>
			)}
		</>
	);
}
