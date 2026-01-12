import { JSX, splitProps } from "solid-js";

import { IconChevronLeft } from "./icons/chevron-left";
import { IconChevronRight } from "./icons/chevron-right";
import { BlazinglyFastLink } from "./link";

export function Pagination(props: { prevHref?: string; nextHref?: string }) {
	return (
		<div class="pointer-events-auto flex items-center gap-2 px-3 py-2">
			<PaginationPrev href={props.prevHref} />
			<PaginationNext href={props.nextHref} />
		</div>
	);
}

export function PaginationLink(allProps: { href?: string; children: JSX.Element; class?: string }) {
	const [props, rest] = splitProps(allProps, ["class"]);

	let _class =
		"bg-gray-1 border-gray-5 focus flex items-center justify-center rounded-full border select-none aria-disabled:opacity-40 aria-disabled:cursor-not-allowed";
	if (props.class) {
		_class += " " + props.class;
	}

	return <BlazinglyFastLink class={_class} {...rest} />;
}

export function PaginationNext(props: { class?: string; href?: string }) {
	return (
		<PaginationLink
			href={props.href}
			class={"py-2 ps-3 pe-2" + (props.class ? " " + props.class : "")}
		>
			<span class="me-1 text-xs">next</span>
			<IconChevronRight />
		</PaginationLink>
	);
}

export function PaginationPrev(props: { class?: string; href?: string }) {
	return (
		<PaginationLink
			href={props.href}
			class={"py-2 ps-2 pe-3" + (props.class ? " " + props.class : "")}
		>
			<IconChevronLeft />
			<span class="ms-1 text-xs">prev</span>
		</PaginationLink>
	);
}

export function NavPaginationLinks(props: { nextHref?: string; prevHref?: string }) {
	return (
		<div class="pointer-events-auto -me-5 flex items-center">
			<PaginationPrev href={props.prevHref} class="border-none" />
			<PaginationNext href={props.nextHref} class="border-none" />
		</div>
	);
}

export function buildPaginatedHref(
	cursorParam: "left" | "right",
	cursorValue: string | null | undefined,
	href: string,
	prevSearchParams: Record<string, string>
) {
	if (!cursorValue) return undefined;
	const newParams = new URLSearchParams({
		...prevSearchParams,
		[cursorParam]: cursorValue,
	});

	if (cursorParam === "left") newParams.delete("right");
	else if (cursorParam === "right") newParams.delete("left");

	return `${href}?${newParams.toString()}`;
}
