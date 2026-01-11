import { useMatch, useNavigate } from "@solidjs/router";
import { JSX, splitProps } from "solid-js";


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
	const navigate = useNavigate();

	return (
		<a
			href={props.href}
			class={"inline-flex px-3 py-4 sm:py-2" + (match() ? " bg-gray-a2" : "")}
			onClick={(e) => {
				// onClick has to be cancelled:
				// for example on transaction pagination, users paginating
				// without moving the mouse will end up more pages ahead
				// than intended
				// - user presses the link -> onMouseDown -> navigation
				// - pagination happens, actual anchor tag changes while
				//   the mouse button is still pressed
				// - user lets go of the mouse button on the anchor
				//   -> onClick triggers on the new anchor tag
				//   triggering another pagination
				// - user is now two pages ahead after "clicking" once

				const url = new URL(String(props.href), window.location.href);
				if (
					url.origin === window.location.origin &&
					e.button === 0 &&
					!e.altKey &&
					!e.ctrlKey &&
					!e.metaKey &&
					!e.shiftKey
				) {
					e.preventDefault();
					return false;
				}
			}}
			onMouseDown={(e) => {
				const url = new URL(String(props.href), window.location.href);
				if (
					url.origin === window.location.origin &&
					e.button === 0 &&
					!e.altKey &&
					!e.ctrlKey &&
					!e.metaKey &&
					!e.shiftKey
				) {
					e.preventDefault();
					navigate(props.href);
				}
			}}
			onTouchStart={(e) => {
				const url = new URL(String(props.href), window.location.href);
				if (url.origin === window.location.origin) {
					e.preventDefault();
					navigate(props.href);
				}
			}}
			onKeyUp={(e) => {
				if (e.key !== "Enter" && e.key !== "Space") return;
				const url = new URL(String(props.href), window.location.href);
				if (url.origin === window.location.origin) {
					e.preventDefault();
					navigate(props.href);
				}
			}}
		>
			{props.children}
		</a>
	);
}
