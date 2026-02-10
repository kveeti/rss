import { useMutation } from "@tanstack/solid-query";

import { API_BASE_URL } from "../lib/constants";
import { Button } from "./button";
import { IconUpload } from "./icons/upload";

export function OpmlExportSection() {
	const exportMutation = useMutation(() => ({
		mutationFn: async () => {
			const response = await fetch(`${API_BASE_URL}/v1/feeds/export`);

			if (!response.ok) {
				throw new Error(`Export failed: ${response.statusText}`);
			}

			const blob = await response.blob();
			const url = URL.createObjectURL(blob);

			const date = new Date().toISOString().split("T")[0];
			const filename = `feeds-${date}.opml`;

			const a = document.createElement("a");
			a.href = url;
			a.download = filename;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);

			URL.revokeObjectURL(url);
		},
	}));

	return (
		<Button
			onClick={() => exportMutation.mutate()}
			isLoading={exportMutation.isPending}
			disabled={exportMutation.isPending}
			variant="ghost"
			size="icon"
			title="Export OPML"
		>
			<IconUpload />
		</Button>
	);
}
