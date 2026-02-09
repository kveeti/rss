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
		queryKey: ["unread-entries", params.leftId, params.rightId],
		queryFn: () => fetchUnreadEntries(params),
	};
}

export function preloadsUnreadPage(props: { search: string }) {
	import("./unread-page");
	const newSearchParams = new URLSearchParams(props.search);
	fetchUnreadEntries({
		leftId: newSearchParams.get("left") ?? undefined,
		rightId: newSearchParams.get("right") ?? undefined,
	});
}
