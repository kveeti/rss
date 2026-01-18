import { JSX } from "solid-js";

export function Empty(props: { children: JSX.Element }) {
	return <p class="bg-gray-a3/60 p-4">{props.children}</p>;
}
