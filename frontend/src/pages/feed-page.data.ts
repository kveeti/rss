import { query } from "@solidjs/router";

import { api } from "../lib/api";

export type FeedWithEntryCounts = {
	id: string;
	title: string;
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
	has_icon: boolean;
	comments_url: string | null;
	published_at: string | null;
	entry_updated_at: string | null;
};

export const getFeed = query((feedId: string) => {
	return api<FeedWithEntryCounts>({
		path: `/v1/feeds/${feedId}`,
		method: "GET",
	});
}, "feed");

export const getFeedEntries = query(
	({
		feedId,
		limit,
		left,
		right,
	}: {
		feedId: string;
		limit?: string;
		left?: string;
		right?: string;
	}) => {
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
	"feedEntries"
);

export function preloadsFeedPage(feedId: string) {
	getFeed(feedId);
	getFeedEntries({ feedId });
}
