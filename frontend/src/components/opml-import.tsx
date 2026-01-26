import { For, Ref, Show, createSignal, onCleanup } from "solid-js";

import { api } from "../lib/api";
import { API_BASE_URL } from "../lib/constants";
import { prettifyUrl } from "../lib/urls";
import { Button } from "./button";
import { Input } from "./input";

type ImportProgress = {
	job_id: string;
	status: string;
	total: number;
	imported: number;
	skipped: number;
	failed: number;
	done: boolean;
	recent: { feed_url: string; status: string; error?: string | null }[];
};

type ImportState =
	| { status: "idle"; loading: false }
	| { status: "starting"; loading: true }
	| { status: "streaming"; loading: false; progress: ImportProgress }
	| { status: "done"; loading: false; progress: ImportProgress }
	| { status: "error"; loading: false; error: string };

type FailedItem = { feed_url: string; error?: string | null };

const failedItemKey = (item: FailedItem) => `${item.feed_url}::${item.error ?? ""}`;

export function OpmlImportSection() {
	let importInputRef: Ref<HTMLInputElement>;
	let importSource: EventSource | undefined;

	const [importState, setImportState] = createSignal<ImportState>({
		status: "idle",
		loading: false,
	});
	const [failedItems, setFailedItems] = createSignal<FailedItem[]>([]);

	async function onImportSubmit(event: SubmitEvent) {
		event.preventDefault();
		setImportState({ status: "starting", loading: true });
		setFailedItems([]);

		// @ts-expect-error
		const file = importInputRef?.files?.[0];
		if (!file) {
			setImportState({ status: "error", loading: false, error: "Select an OPML file." });
			return;
		}

		if (importSource) {
			importSource.close();
			importSource = undefined;
		}

		const body = new FormData();
		body.append("file", file);

		try {
			const res = await api<{
				status: "import_started";
				job_id: string;
				total: number;
				skipped: number;
			}>({
				path: "/v1/feeds/import",
				method: "POST",
				body,
			});

			const initialProgress: ImportProgress = {
				job_id: res.job_id,
				status: "running",
				total: res.total,
				imported: 0,
				skipped: res.skipped,
				failed: 0,
				done: false,
				recent: [],
			};

			setImportState({
				status: "streaming",
				loading: false,
				progress: initialProgress,
			});

			// @ts-expect-error
			importInputRef.value = "";

			importSource = new EventSource(`${API_BASE_URL}/v1/feeds/import/${res.job_id}/events`);
			importSource.onmessage = (event) => {
				try {
					const payload = JSON.parse(event.data) as ImportProgress;
					trackFailures(payload);
					if (payload.done) {
						importSource?.close();
						importSource = undefined;
						setImportState({ status: "done", loading: false, progress: payload });
					} else {
						setImportState({ status: "streaming", loading: false, progress: payload });
					}
				} catch {
					setImportState({
						status: "error",
						loading: false,
						error: "Failed to parse import progress.",
					});
				}
			};
			importSource.onerror = () => {
				importSource?.close();
				importSource = undefined;
				setImportState({
					status: "error",
					loading: false,
					error: "Import connection lost.",
				});
			};
		} catch (error) {
			setImportState({
				status: "error",
				loading: false,
				error: error instanceof Error ? error.message : "Import failed.",
			});
		}
	}

	onCleanup(() => {
		importSource?.close();
	});

	const importProgress = () =>
		importState().status === "streaming" || importState().status === "done"
			? importState().progress
			: null;

	const trackFailures = (payload: ImportProgress) => {
		const failures = payload.recent.filter((item) => item.status === "failed");
		if (!failures.length) {
			return;
		}

		setFailedItems((previous) => {
			const existing = new Set(previous.map(failedItemKey));
			const merged = [...previous];
			for (const failure of failures) {
				const entry: FailedItem = {
					feed_url: failure.feed_url,
					error: failure.error ?? null,
				};
				const key = failedItemKey(entry);
				if (!existing.has(key)) {
					merged.push(entry);
					existing.add(key);
				}
			}
			return merged;
		});
	};

	return (
		<section class="border-gray-a5 mt-12 border-t pt-6">
			<h2 class="mb-4 text-lg leading-none">Import OPML</h2>
			<form class="space-y-4" onSubmit={onImportSubmit}>
				<Input
					label="OPML file"
					type="file"
					name="opml"
					accept=".opml,.xml,text/xml"
					ref={
						// @ts-expect-error
						importInputRef
					}
					required
				/>

				<div class="flex justify-end">
					<Button
						type="submit"
						isLoading={importState().loading}
						disabled={
							importState().status === "starting" ||
							importState().status === "streaming"
						}
					>
						Import
					</Button>
				</div>
			</form>

			<Show when={importProgress()}>
				{(progress) => (
					<div class="mt-4 text-sm" role="status" aria-live="polite">
						<p>
							Imported {progress().imported} of {progress().total}
						</p>
						{progress().skipped > 0 && <p>Skipped {progress().skipped}</p>}
						{progress().failed > 0 && <p>Failed {progress().failed}</p>}
						<Show when={importState().status === "done" || progress().done}>
							<p class="bg-green-a5 border-green-a6 mt-1 border p-4">
								Import complete.
							</p>
						</Show>
						<Show when={importState().status !== "done" && !progress().done}>
							<p class="text-gray-11">Import in progress...</p>
						</Show>

						<Show when={failedItems().length > 0}>
							<div class="border-red-a6 bg-red-a3 mt-3 border p-2">
								<p class="text text-xs">Failed feeds ({failedItems().length})</p>
								<ul class="mt-2 max-h-40 space-y-1 overflow-y-auto pr-2 text-xs">
									<For each={failedItems()}>
										{(item) => (
											<li>
												{prettifyUrl(item.feed_url)}
												{item.error ? ` (${item.error})` : ""}
											</li>
										)}
									</For>
								</ul>
							</div>
						</Show>
					</div>
				)}
			</Show>

			<Show when={importState().status === "error"}>
				<p class="bg-red-a3 border-red-a6 mt-4 border p-2 text-sm">{importState().error}</p>
			</Show>
		</section>
	);
}
