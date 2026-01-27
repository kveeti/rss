import { Show, createSignal } from "solid-js";

import { API_BASE_URL } from "../lib/constants";

const FALLBACK_ICON_COLORS = [
	"#3B82F6",
	"#22C55E",
	"#F59E0B",
	"#EF4444",
	"#8B5CF6",
	"#06B6D4",
	"#F97316",
	"#10B981",
	"#6366F1",
	"#EAB308",
	"#EC4899",
	"#14B8A6",
	"#84CC16",
	"#0EA5E9",
	"#A855F7",
	"#F43F5E",
	"#16A34A",
	"#FACC15",
	"#2563EB",
	"#FB7185",
	"#0D9488",
	"#7C3AED",
	"#0284C7",
	"#D946EF",
	"#4F46E5",
	"#F87171",
];

const FALLBACK_ICON_NEUTRAL = "#64748B";

export function FeedIcon(props: {
	class?: string;
	feedId?: string;
	hasIcon?: boolean;
	fallbackUrl?: string;
}) {
	const [imgFailed, setImgFailed] = createSignal(false);

	const feedId = () => props.feedId;
	const hasIcon = () => props.hasIcon;
	const fallbackUrl = () => props.fallbackUrl;

	const initial = () => getHostnameInitial(fallbackUrl());
	const backgroundColor = () => getHostnameColor(initial());

	const showImage = () => Boolean(feedId()) && hasIcon() !== false && !imgFailed();

	return (
		<Show
			when={showImage()}
			fallback={
				<span
					class={
						"inline-flex items-center justify-center text-[0.65em] leading-none font-semibold text-white" +
						(props.class ? ` ${props.class}` : "")
					}
					style={{ "background-color": backgroundColor() }}
					aria-hidden="true"
				>
					{initial()}
				</span>
			}
		>
			<img
				class={props.class}
				src={API_BASE_URL + `/v1/feeds/${feedId()}/icon`}
				aria-hidden="true"
				onError={() => setImgFailed(true)}
			/>
		</Show>
	);
}

function getHostnameInitial(value?: string) {
	if (!value) return "";

	let hostname = value;
	try {
		hostname = new URL(value).hostname;
	} catch {
		hostname = new URL(`https://${value}`).hostname;
	}

	const trimmed = hostname.replace(/^www\./, "").trim();
	return trimmed ? trimmed[0]!.toUpperCase() : "?";
}

function getHostnameColor(hostname: string) {
	const initial = getHostnameInitial(hostname);
	const code = initial.toUpperCase().charCodeAt(0);
	if (code < 65 || code > 90) return FALLBACK_ICON_NEUTRAL;

	return FALLBACK_ICON_COLORS[code - 65] ?? FALLBACK_ICON_NEUTRAL;
}
