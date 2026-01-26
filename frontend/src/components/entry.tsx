import { FeedIcon } from "./feed-icon";
import { IconDividerVertical } from "./icons/divider-vertical";

type Falsy<T> = T | null | undefined | false;

export function Entry(props: {
	title: string;
	date: Falsy<Date>;
	url: string;
	commentsUrl: Falsy<string>;
	feed: {
		id: string;
		has_icon: boolean;
		feed_url: string;
		site_url: string | null;
	};
}) {
	return (
		<li class="group/entry focus:bg-gray-a2 hover:bg-gray-a2 group/feed relative flex flex-col gap-2 p-3 select-none">
			<a
				href={props.url}
				target="_blank"
				class="focus2 absolute top-0 left-0 h-full w-full"
			></a>

			<div class="flex gap-3">
				<div class="font-cool flex h-[1lh] flex-shrink-0 items-center justify-center text-[1.3rem]">
					<FeedIcon class="size-6" feed={props.feed} />
				</div>

				<div class="flex flex-col gap-1">
					<span class="font-cool inline text-[1.3rem] select-auto group-hover/feed:underline group-has-[a[id=comments]:hover]/feed:no-underline">
						{props.title}
					</span>

					<div class="flex items-center gap-1">
						{props.date && (
							<p class="text-gray-11 text-sm">{relativeTime(props.date)}</p>
						)}

						{props.date && props.commentsUrl && <IconDividerVertical />}

						{props.commentsUrl && (
							<a
								id="comments"
								href={props.commentsUrl}
								target="_blank"
								class="group/comments text-gray-11 relative -m-4 p-4 text-sm underline outline-none"
							>
								<span class="in-focus:outline-gray-a10 group-hover/comments:text-white in-focus:outline-2 in-focus:outline-offset-2 in-focus:outline-none in-focus:outline-solid">
									comments
								</span>
							</a>
						)}
					</div>
				</div>
			</div>
		</li>
	);
}

export function EntryIcon(props: {
	feed: { id: string; has_icon: boolean; feed_url: string; site_url: string | null };
}) {
	return (
		<div class="flex h-[1lh] flex-shrink-0 items-center justify-center text-[1.3rem]">
			<FeedIcon class="size-6" feed={props.feed} />
		</div>
	);
}

export function EntryDate(props: { date?: string | false | null }) {
	if (!props.date) return null;

	const date = new Date(props.date);
	const dateFormatted = relativeTime(date);

	return <p class="text-gray-11 text-sm">{dateFormatted}</p>;
}

const rtf = new Intl.RelativeTimeFormat("en", { numeric: "auto" });
const unitsInSec = [60, 3600, 86400, 86400 * 7, 86400 * 30, 86400 * 365, Infinity];
const unitStrings = ["second", "minute", "hour", "day", "week", "month", "year"] as const;

function relativeTime(date: Date) {
	const secondsDiff = Math.round((date.getTime() - Date.now()) / 1000);
	const unitIndex = unitsInSec.findIndex((cutoff) => cutoff > Math.abs(secondsDiff));
	const divisor = unitIndex ? unitsInSec[unitIndex - 1] : 1;

	return rtf.format(Math.floor(secondsDiff / divisor), unitStrings[unitIndex]);
}
