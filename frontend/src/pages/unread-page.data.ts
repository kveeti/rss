import { queryEntries } from "./entries-page.data";

export function getUnreadEntries(leftId: string | undefined, rightId: string | undefined) {
	return queryEntries({
		left: leftId,
		right: rightId,
		unread: "true",
	});
}

export function preloadsUnreadPage(props: { search: string }) {
	import("./unread-page");
	const newSearchParams = new URLSearchParams(props.search);
	getUnreadEntries(
		newSearchParams.get("left") ?? undefined,
		newSearchParams.get("right") ?? undefined
	);
}
