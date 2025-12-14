import { JSX } from "solid-js";

const baseInputStyles = "focus border border-gray-a6 p-2 sm:text-sm text-base";

export function Input(props: JSX.InputHTMLAttributes<HTMLInputElement> & { label: string }) {
	if (props.class) {
		props.class += " " + baseInputStyles;
	}

	return (
		<div class="flex flex-col gap-2">
			<label class="text-gray-11 text-sm">{props.label}</label>
			<input class={baseInputStyles} {...props} />
		</div>
	);
}
