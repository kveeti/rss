import { API_BASE_URL } from "../lib/constants";

export function FeedIcon(props: { class?: string; feedId: string }) {
	return (
		<img
			class={props.class}
			src={API_BASE_URL + `/v1/feeds/${props.feedId}/icon`}
			aria-hidden="true"
		/>
	);
}
