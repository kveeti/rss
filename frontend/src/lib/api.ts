import { API_BASE_URL } from "./constants";

export type ApiError = {
	error: string;
	details: Record<string, string>;
};

type Props = {
	path: string;
	method: string;
	body?: unknown;
	query?: Record<string, string>;
	signal?: AbortSignal;
};

export async function api<TReturnValue>(props: Props) {
	const fetchProps = {
		signal: props.signal,
		method: props.method ?? "GET",
	} as RequestInit;

	if (props.body instanceof FormData) {
		fetchProps.body = props.body;
	} else if (props.body) {
		fetchProps.body = JSON.stringify(props.body);
		fetchProps.headers = { "Content-Type": "application/json" };
	}

	return fetch(
		`${API_BASE_URL}${props.path}${props.query ? `?${new URLSearchParams(props.query)}` : ""}`,
		fetchProps
	)
		.catch(() => {
			throw new Error("network error");
		})
		.then(async (res) => {
			const json = await res.json().catch(() => null);

			if (res.ok) {
				return json as TReturnValue;
			} else {
				throw new Error(json?.error ?? `unexpected server error - status: ${res.status}`);
			}
		});
}
