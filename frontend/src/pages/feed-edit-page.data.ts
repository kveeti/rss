import { getFeed } from "./feed-page.data";

export function preloadsFeedEditPage(feedId: string) {
	import("./feed-edit-page");
	getFeed(feedId);
}
