import { query } from "@solidjs/router";

import { api } from "../lib/api";

export type Feed = {
	id: string;
	title: string;
	feed_url: string;
	site_url: string;
	created_at: string;
	entry_count: number;
	unread_entry_count: number;
};

export const getFeeds = query(() => {
	return api<Array<Feed>>({
		path: "/v1/feeds",
		method: "GET",
	});
}, "feeds");

export function preloadsFeedsPage() {
	getFeeds();
}
