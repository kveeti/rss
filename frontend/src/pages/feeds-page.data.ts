import { query } from "@solidjs/router";

import { api } from "../lib/api";
import { FeedWithEntryCounts } from "./feed-page.data";

export const getFeeds = query(() => {
	return api<Array<FeedWithEntryCounts>>({
		path: "/v1/feeds",
		method: "GET",
	});
}, "feeds");

export function preloadsFeedsPage() {
	getFeeds();
}
