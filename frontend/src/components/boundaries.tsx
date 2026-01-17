import { useIsRouting } from "@solidjs/router";
import { ErrorBoundary, JSX, Show, Suspense } from "solid-js";

export function Boundaries(props: {
	error: (reset: () => void) => JSX.Element;
	loading: JSX.Element;
	children: JSX.Element;
}) {
	const isRouting = useIsRouting();

	return (
		<ErrorBoundary fallback={(_error, reset) => props.error(reset)}>
			<Suspense fallback={props.loading}>
				<Show when={!isRouting()}>{props.children}</Show>
			</Suspense>
		</ErrorBoundary>
	);
}
