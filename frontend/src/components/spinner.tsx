import { JSX } from "solid-js";
import { Show } from "solid-js";

import c from "./spinner.module.css";

export function ConditionalSpinner(props: { children: JSX.Element; isLoading?: boolean }) {
	return (
		<>
			<Show when={!props.isLoading}>{props.children}</Show>

			<Show when={props.isLoading}>
				<span class={c.wrapper}>
					<span aria-hidden="true" class={c.childrenWrapper}>
						{props.children}
					</span>

					<span class={c.spinnerWrapper}>
						<Spinner />
					</span>
				</span>
			</Show>
		</>
	);
}

export function Spinner() {
	return (
		<span class={c.spinner}>
			<span class={c.leaf} />
			<span class={c.leaf} />
			<span class={c.leaf} />
			<span class={c.leaf} />
			<span class={c.leaf} />
			<span class={c.leaf} />
			<span class={c.leaf} />
			<span class={c.leaf} />
		</span>
	);
}
