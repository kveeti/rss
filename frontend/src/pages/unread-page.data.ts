import { fetchEntries } from "./entries-page.data";

export type UnreadEntriesParams = {
	leftId: string | undefined;
	rightId: string | undefined;
};

export async function fetchUnreadEntries({ leftId, rightId }: UnreadEntriesParams) {
	return fetchEntries({
		left: leftId,
		right: rightId,
		unread: "true",
	});
}

export function unreadEntriesQueryOptions(params: UnreadEntriesParams) {
	return {
		queryKey: ["entries", "unread", params.leftId, params.rightId],
		queryFn: () => fetchUnreadEntries(params),
	};
}

export async function prefetchUnreadPage(queryClient: any, props: { search: string }) {
	const searchParams = new URLSearchParams(props.search);
	const params: UnreadEntriesParams = {
		leftId: searchParams.get("left") ?? undefined,
		rightId: searchParams.get("right") ?? undefined,
	};

	await queryClient.prefetchQuery(unreadEntriesQueryOptions(params));
	import("./unread-page");
}
