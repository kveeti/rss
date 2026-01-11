import { useMatch } from "@solidjs/router";
import { JSX, splitProps } from "solid-js";

import { BlazinglyFastLink } from "./components/link";

export function Page(allProps: { class?: string; children: JSX.Element }) {
	const [props, rest] = splitProps(allProps, ["class"]);

	let _class = "mt-44 mb-44 sm:mt-14";
	if (props.class) {
		_class += " " + props.class;
	}

	return <div class={_class} {...rest} />;
}

export function NavWrap(allProps: { class?: string; children: JSX.Element }) {
	const [props, rest] = splitProps(allProps, ["class"]);

	let _class =
		"bg-gray-1 border-gray-a5 fixed right-0 bottom-0 left-0 z-10 border-t sm:top-0 sm:bottom-[unset] sm:border-0 w-full";
	if (props.class) {
		_class += " " + props.class;
	}

	return <div class={_class} {...rest} />;
}

export function Nav(allProps: { class?: string; children: JSX.Element }) {
	const [props, rest] = splitProps(allProps, ["class"]);

	let _class = "pwa:pb-12 pwa:px-8 mx-auto flex max-w-160  px-3 w-full";
	if (props.class) {
		_class += " " + props.class;
	}

	return <div class={_class} {...rest} />;
}

export function DefaultNavLinks() {
	return (
		<ul class="flex select-none">
			<li>
				<NavLink href="/unread">unread</NavLink>
			</li>

			<li>
				<NavLink href="/feeds">feeds</NavLink>
			</li>

			<li>
				<NavLink href="/feeds/new">new feed</NavLink>
			</li>

			<li>
				<NavLink href="/entries">entries</NavLink>
			</li>
		</ul>
	);
}

function NavLink(props: { children: JSX.Element; href: string }) {
	const match = useMatch(() => props.href.split("?")[0]!);

	return (
		<BlazinglyFastLink
			{...props}
			class={"inline-flex px-3 py-4 sm:py-2" + (match() ? " bg-gray-a3" : "")}
		/>
	);
}
