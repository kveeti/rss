import { useNavigate } from "@solidjs/router";
import { useMutation, useQueryClient } from "@tanstack/solid-query";
import { For, Show, createSignal } from "solid-js";

import { Button, buttonStyles } from "../components/button";
import { IconInfo } from "../components/icons/info";
import { Input } from "../components/input";
import { OpmlImportSection } from "../components/opml-import";
import { DefaultNavLinks, Nav, NavWrap, Page } from "../layout";
import { api } from "../lib/api";

type States =
	| {
			phase: "discovered_multiple";
			feed_urls: string[];
			similar_feed_url?: string;
			status: "idle";
			loading: false;
	  }
	| {
			phase: "init";
			loading: true;
			similar_feed_url?: never;
	  }
	| {
			phase: "init";
			loading: false;
			error: string;
			similar_feed_url?: never;
	  }
	| {
			phase: "init";
			loading: false;
			similar_feed_url?: never;
	  }
	| {
			phase: "only_one_similar_feed";
			similar_feed_url: string;
			loading: false;
	  };

export default function NewFeedPage() {
	let inputRef!: HTMLInputElement;
	let formRef!: HTMLFormElement;
	const queryClient = useQueryClient();

	const [state, setState] = createSignal<States>({ phase: "init", loading: false });
	const navigate = useNavigate();

	const addFeedMutation = useMutation(() => ({
		mutationFn: async ({
			url,
			force_similar_feed,
		}: {
			url: string;
			force_similar_feed?: boolean;
		}) => {
			return api<
				| { status: "feed_added" }
				| { status: "discovered_multiple"; feed_urls: string[]; similar_feed_url?: string }
				| { status: "duplicate_feed" }
				| { status: "similar_feed"; similar_feed_url: string }
			>({
				path:
					"/v1/feeds?url=" +
					encodeURIComponent(url) +
					"&force_similar_feed=" +
					(force_similar_feed ?? "false"),
				method: "POST",
			});
		},
		onSuccess: (res) => {
			if (res.status === "feed_added") {
				queryClient.invalidateQueries({ queryKey: ["feeds"] });
				navigate("/feeds");
			} else if (res.status === "discovered_multiple") {
				setState({
					phase: "discovered_multiple",
					status: "idle",
					feed_urls: res.feed_urls,
					similar_feed_url: res.similar_feed_url,
					loading: false,
				});
			} else if (res.status === "similar_feed") {
				setState({
					phase: "only_one_similar_feed",
					similar_feed_url: res.similar_feed_url,
					loading: false,
				});
			}
		},
		onError: (error: Error) => {
			setState({
				phase: "init",
				loading: false,
				error: error.message,
			});
		},
	}));

	async function onSubmit(event: SubmitEvent) {
		event.preventDefault();

		addFeed(
			// @ts-expect-error
			event.target.url.value,
			state().phase === "only_one_similar_feed"
		);
	}

	async function addFeed(url: string, force_similar_feed?: boolean) {
		setState({
			phase: "init",
			loading: true,
		});

		addFeedMutation.mutate({ url, force_similar_feed });
	}

	return (
		<>
			<NavWrap>
				<Nav>
					<DefaultNavLinks />
				</Nav>
			</NavWrap>

			<Page>
				<main class="mx-auto max-w-[20rem]">
					<h1 class="font-cool mt-4 mb-8 text-3xl font-medium">New Feed</h1>

					<form class="space-y-4" onSubmit={onSubmit} ref={formRef}>
						<div class="flex flex-col gap-2">
							<Input label="URL" type="text" name="url" ref={inputRef} required />
						</div>

						<div class="flex justify-end">
							<Button type="submit" isLoading={state().loading}>
								{state().phase === "only_one_similar_feed"
									? "Add anyway"
									: "Submit"}
							</Button>
						</div>
					</form>

					<Show when={state().similar_feed_url}>
						<div class="border-gray-a2 border-gray-a9 mt-4 flex gap-2 border-l-3 py-1 pl-2">
							<div class="size-6">
								<IconInfo />
							</div>

							<div>
								<p class="mb-2 text-sm">
									<b>NOTE:</b> Feed with similar URL already saved:
								</p>
								<code>{state().similar_feed_url}</code>
							</div>
						</div>
					</Show>

					<Show when={state().phase === "discovered_multiple"}>
						<DiscoveredMultiple
							feed_urls={
								// @ts-expect-error
								state().feed_urls
							}
							onClick={(url) => {
								inputRef.value = url;
							}}
						/>
					</Show>

					<OpmlImportSection />
				</main>
			</Page>
		</>
	);
}

function DiscoveredMultiple(props: { feed_urls: string[]; onClick: (url: string) => void }) {
	return (
		<div class="mt-8 flex flex-col">
			<h2 class="mb-4 text-lg leading-none">Found multiple feeds</h2>

			<ul class="flex flex-col gap-1">
				<For each={props.feed_urls}>
					{(feed_url) => (
						<li>
							<button
								class="focus hover:bg-gray-a2 -mx-2 p-2"
								onClick={() => props.onClick(feed_url)}
							>
								{feed_url}
							</button>
						</li>
					)}
				</For>
			</ul>
		</div>
	);
}
