import { api } from "../lib/api";
import { FeedWithEntryCounts } from "./feed-page.data";

export function feedsQueryOptions() {
	return {
		queryKey: ["feeds"],
		queryFn: async () => {
			return api<Array<FeedWithEntryCounts>>({
				path: "/v1/feeds",
				method: "GET",
			});
		},
	};
}

export async function prefetchFeedsPage(queryClient: any) {
	await queryClient.prefetchQuery(feedsQueryOptions());
	import("./feeds-page");
}
