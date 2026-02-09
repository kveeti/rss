import { feedQueryOptions } from "./feed-page.data";

export async function prefetchFeedEditPage(queryClient: any, feedId: string) {
	await queryClient.prefetchQuery(feedQueryOptions(feedId));
	import("./feed-edit-page");
}
