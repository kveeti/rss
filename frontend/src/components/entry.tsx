import { createMutation, useQueryClient } from "@tanstack/solid-query";
import {
	type Accessor,
	type JSX,
	type ParentProps,
	createContext,
	createSignal,
	useContext,
} from "solid-js";

import { api } from "../lib/api";
import { FeedIcon } from "./feed-icon";
import { IconDividerVertical } from "./icons/divider-vertical";

type FeedEntry = {
	id: string;
	title: string;
	url: string;
	feed_id: string;
	comments_url: string | null;
	read_at: string | null;
	published_at: string | null;
	entry_updated_at: string | null;
};

type EntryWithIcon = FeedEntry & { has_icon?: boolean };

const EntryContext = createContext<Accessor<EntryWithIcon>>();

function useEntry() {
	const entry = useContext(EntryContext);
	if (!entry) {
		throw new Error("Entry sub-components must be used within Entry.Root");
	}
	return entry;
}

type EntryRootProps = {
	entry: EntryWithIcon;
	children: JSX.Element;
};

function EntryRoot(props: EntryRootProps) {
	return (
		<EntryContext.Provider value={() => props.entry}>
			<li class="group/entry focus:bg-gray-a2 hover:bg-gray-a2 group/feed relative flex flex-col gap-2 p-3 select-none">
				<a
					href={props.entry.url}
					target="_blank"
					class="focus2 absolute top-0 left-0 h-full w-full"
				/>
				{props.children}
			</li>
		</EntryContext.Provider>
	);
}

function EntryIcon() {
	const entry = useEntry();
	const hasIcon = () => entry().has_icon;

	return (
		<div class="font-cool flex h-[1lh] flex-shrink-0 items-center justify-center text-[1.3rem]">
			<a href={`/feeds/${entry().feed_id}`} class="relative z-10 -m-4 p-4" id="overlaybutton">
				<FeedIcon
					class="size-6"
					feedId={entry().feed_id}
					hasIcon={hasIcon()}
					fallbackUrl={entry().url}
				/>
			</a>
		</div>
	);
}

function EntryContent(props: ParentProps) {
	return <div class="flex flex-1 flex-col gap-1">{props.children}</div>;
}

function EntryTitle() {
	const entry = useEntry();
	return (
		<span class="font-cool inline text-[1.3rem] select-auto group-hover/feed:underline group-has-[[id=overlaybutton]:hover]/feed:no-underline">
			{entry().title}
		</span>
	);
}

function EntryMeta(props: ParentProps) {
	return (
		<div class="flex items-center gap-1">
			{Array.isArray(props.children)
				? props.children.map((c, i) => (
						<>
							{i != 0 && <EntryDivider />}
							{c}
						</>
					))
				: props.children}
		</div>
	);
}

function EntryDate() {
	const entry = useEntry();
	const date = () => {
		const dateStr = entry().published_at || entry().entry_updated_at;
		return dateStr ? new Date(dateStr) : null;
	};

	const dateValue = date();
	if (!dateValue) return null;

	return <p class="text-gray-11 text-sm">{relativeTime(dateValue)}</p>;
}

function EntryDivider() {
	return <IconDividerVertical />;
}

function EntryComments() {
	const entry = useEntry();
	const commentsUrl = () => entry().comments_url;

	if (!commentsUrl()) return null;
	return (
		<a
			id="overlaybutton"
			href={commentsUrl()!}
			target="_blank"
			class="group/comments text-gray-11 relative -m-4 p-4 text-sm underline outline-none"
		>
			<span class="in-focus:outline-gray-a10 group-hover/comments:text-white in-focus:outline-2 in-focus:outline-offset-2 in-focus:outline-none in-focus:outline-solid">
				comments
			</span>
		</a>
	);
}

type EntryReadToggleProps = {
	onReadChange?: (id: string, read: boolean) => void;
};

function EntryReadToggle(props: EntryReadToggleProps) {
	const entry = useEntry();
	const [optimisticRead, setOptimisticRead] = createSignal<boolean | null>(null);
	const [isUpdating, setIsUpdating] = createSignal(false);
	const queryClient = useQueryClient();

	const isRead = () => {
		const optimistic = optimisticRead();
		if (optimistic !== null) return optimistic;
		return !!entry().read_at;
	};

	const toggleReadMutation = createMutation(() => ({
		mutationFn: async ({ id, read }: { id: string; read: boolean }) => {
			return api<{ success: boolean }>({
				method: "POST",
				path: `/v1/entries/${id}/read`,
				body: { read },
			});
		},
		onSuccess: (_data, variables) => {
			queryClient.invalidateQueries({ queryKey: ["entries"] });

			props.onReadChange?.(variables.id, variables.read);
		},
		onError: (
			_error,
			variables,
			context: { previousOptimistic: boolean | null } | undefined
		) => {
			setOptimisticRead(context?.previousOptimistic ?? null);
		},
		mutationKey: ["toggle-read", entry().id],
	}));

	async function handleReadClick(e: MouseEvent) {
		e.preventDefault();
		e.stopPropagation();

		if (isUpdating()) return;

		const newReadState = !isRead();
		const previousOptimistic = optimisticRead();

		setOptimisticRead(newReadState);
		setIsUpdating(true);

		props.onReadChange?.(entry().id, newReadState);

		try {
			toggleReadMutation.mutate(
				{ id: entry().id, read: newReadState },
				{
					onSettled: () => {
						setIsUpdating(false);
					},
					// @ts-expect-error - context typing issue
					context: { previousOptimistic },
				}
			);
		} catch (error) {
			setOptimisticRead(previousOptimistic);
			console.error("Failed to update read status:", error);
			setIsUpdating(false);
		}
	}

	return (
		<button
			id="overlaybutton"
			onClick={handleReadClick}
			disabled={isUpdating()}
			class="group/read text-gray-11 relative -m-4 p-4 text-sm underline"
		>
			<span class="in-focus:outline-gray-a10 group-hover/read:text-white in-focus:outline-2 in-focus:outline-offset-2 in-focus:outline-none in-focus:outline-solid">
				{isRead() ? "mark unread" : "mark read"}
			</span>
		</button>
	);
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

export function formatDate(dateStr: string | false | null | undefined) {
	if (!dateStr) return null;
	const date = new Date(dateStr);
	return relativeTime(date);
}

export const Entry = {
	Root: EntryRoot,
	Icon: EntryIcon,
	Content: EntryContent,
	Title: EntryTitle,
	Meta: EntryMeta,
	Date: EntryDate,
	Divider: EntryDivider,
	Comments: EntryComments,
	ReadToggle: EntryReadToggle,
};
