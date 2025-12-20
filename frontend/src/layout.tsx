import { useMatch } from "@solidjs/router";
import { JSX } from "solid-js";

export function Layout(props: { children: JSX.Element }) {
	return (
		<>
			<div class="bg-gray-1 border-gray-a5 fixed right-0 bottom-0 left-0 z-10 border-t sm:top-0 sm:bottom-[unset] sm:border-0">
				<div class="pwa:pb-12 pwa:px-8 mx-auto flex max-w-160 justify-center px-3 sm:justify-start">
					<ul class="flex">
						<li>
							<NavLink href="/feeds">feeds</NavLink>
						</li>

						<li>
							<NavLink href="/feeds/new">new feed</NavLink>
						</li>
					</ul>
				</div>
			</div>

			<div class="mt-44 mb-44 sm:mt-14">{props.children}</div>
		</>
	);
}

function NavLink(props: { children: JSX.Element; href: string }) {
	const match = useMatch(() => props.href);

	return (
		<a
			href={props.href}
			class={"inline-flex px-3 py-4 sm:py-2" + (match() ? " bg-gray-a2" : "")}
		>
			{props.children}
		</a>
	);
}
