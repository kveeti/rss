import { useNavigate } from "@solidjs/router";
import { JSX } from "solid-js";

type HTMLAnchorProps = JSX.ButtonHTMLAttributes<HTMLAnchorElement> & {
	children?: JSX.Element;
	href?: string;
};

export const textLinkStyles =
	"focus text-current underline -m-2 p-2 hover:bg-gray-a3 inline-block max-w-max";

export const linkStyles = "focus text-sm text-current underline";

export function Link(
	props: {
		variant?: "default" | "text";
	} & HTMLAnchorProps
) {
	const baseStyles = props.variant === "default" ? linkStyles : textLinkStyles;
	let _class = baseStyles + (props.class ? " " + props.class : "");

	return <BlazinglyFastLink {...props} class={_class} />;
}

export function BlazinglyFastLink(props: HTMLAnchorProps) {
	const navigate = useNavigate();

	return (
		<a
			{...props}
			role="link"
			aria-disabled={props.href ? undefined : "true"}
			onClick={(e) => {
				if (!props.href) return;

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
				if (!props.href) return;

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
				if (!props.href) return;

				const url = new URL(String(props.href), window.location.href);
				if (url.origin === window.location.origin) {
					e.preventDefault();
					navigate(props.href);
				}
			}}
			onKeyUp={(e) => {
				if (!props.href) return;

				if (e.key !== "Enter" && e.key !== "Space") return;
				const url = new URL(String(props.href), window.location.href);
				if (url.origin === window.location.origin) {
					e.preventDefault();
					navigate(props.href);
				}
			}}
		/>
	);
}
