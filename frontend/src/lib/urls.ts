export function prettifyUrl(url: string) {
	if (!url) {
		return url;
	}

	return url
		.replace(/^https?:\/\/www\./, "")
		.replace(/^https?:\/\//, "")
		.replace(/\/$/, "");
}
