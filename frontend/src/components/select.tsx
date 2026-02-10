import { JSX, splitProps } from "solid-js";

import { IconChevronDown } from "./icons/chevron-down";

export function Select(
	allProps: {
		children: JSX.Element;
		value?: string;
	} & JSX.HTMLAttributes<HTMLSelectElement>
) {
	const [split, rest] = splitProps(allProps, ["class"]);

	let _class = "focus border-gray-a6 h-8 appearance-none border ps-2 pe-6.5 outline-none";
	if (split.class) {
		_class += " " + split.class;
	}

	return (
		<div class="relative">
			<select {...rest} class={_class}>
				{rest.children}
			</select>

			<IconChevronDown class="absolute top-2 right-1.5" />
		</div>
	);
}
