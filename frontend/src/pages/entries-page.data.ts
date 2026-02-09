import { api } from "../lib/api";
import { type FeedEntry } from "./feed-page.data";

export type FilterParams = {
	feed_id?: string;
	query?: string;
	left?: string;
	right?: string;
	unread?: string;
	starred?: string;
	start?: string;
	end?: string;
	sort?: string;
};

export type EntriesResponse = {
	entries: Array<FeedEntry & { has_icon: boolean }>;
	next_id: string | null;
	prev_id: string | null;
};

export async function fetchEntries(params: FilterParams): Promise<EntriesResponse> {
	const queryParams: Record<string, string> = {};

	if (params.left) queryParams.left = params.left;
	if (params.right) queryParams.right = params.right;
	if (params.query) queryParams.query = params.query;
	if (params.feed_id) queryParams.feed_id = params.feed_id;
	if (params.unread) queryParams.unread = params.unread;
	if (params.starred) queryParams.starred = params.starred;
	if (params.start) queryParams.start = params.start;
	if (params.end) queryParams.end = params.end;
	if (params.sort) queryParams.sort = params.sort;

	return api<EntriesResponse>({
		method: "GET",
		path: "/v1/entries",
		query: queryParams,
	});
}

export function entriesQueryOptions(params: FilterParams) {
	return {
		queryKey: [
			"entries",
			params.left,
			params.right,
			params.query,
			params.feed_id,
			params.unread,
			params.starred,
			params.start,
			params.end,
			params.sort,
		],
		queryFn: () => fetchEntries(params),
	};
}

export async function prefetchEntriesPage(queryClient: any, props: { search: string }) {
	const searchParams = new URLSearchParams(props.search);
	const params: FilterParams = {
		feed_id: searchParams.get("feed_id") ?? undefined,
		query: searchParams.get("query") ?? undefined,
		left: searchParams.get("left") ?? undefined,
		right: searchParams.get("right") ?? undefined,
		unread: searchParams.get("unread") ?? undefined,
		starred: searchParams.get("starred") ?? undefined,
		start: searchParams.get("start") ?? undefined,
		end: searchParams.get("end") ?? undefined,
		sort: searchParams.get("sort") ?? undefined,
	};

	await queryClient.prefetchQuery(entriesQueryOptions(params));
	import("./entries-page");
}
