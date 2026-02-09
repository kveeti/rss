import { api } from "../lib/api";

export type FeedWithEntryCounts = {
	id: string;
	title: string;
	source_title: string;
	user_title: string | null;
	feed_url: string;
	site_url: string | null;
	created_at: string;
	entry_count: number;
	unread_entry_count: number;
	has_icon: boolean;
	last_synced_at: string | null;
	last_sync_result: string | null;
};

export type FeedEntry = {
	id: string;
	title: string;
	url: string;
	feed_id: string;
	comments_url: string | null;
	read_at: string | null;
	published_at: string | null;
	entry_updated_at: string | null;
};

export function feedQueryOptions(feedId: string) {
	return {
		queryKey: ["entries", "feed", feedId],
		queryFn: async () => {
			return api<FeedWithEntryCounts>({
				path: `/v1/feeds/${feedId}`,
				method: "GET",
			});
		},
	};
}

export function feedEntriesQueryOptions({
	feedId,
	limit,
	left,
	right,
}: {
	feedId: string;
	limit?: string;
	left?: string;
	right?: string;
}) {
	const query: Record<string, string> = {};

	if (limit) {
		query.limit = limit;
	}
	if (left) {
		query.left = left;
	}
	if (right) {
		query.right = right;
	}

	return {
		queryKey: ["feed-entries", feedId, limit, left, right],
		queryFn: async () => {
			return api<{
				entries: Array<FeedEntry>;
				next_id: string;
				prev_id: string;
			}>({
				path: `/v1/feeds/${feedId}/entries`,
				query,
				method: "GET",
			});
		},
	};
}

export async function prefetchFeedPage(queryClient: any, feedId: string) {
	await queryClient.prefetchQuery(feedQueryOptions(feedId));
	await queryClient.prefetchQuery(feedEntriesQueryOptions({ feedId }));
	import("./feed-page");
}
