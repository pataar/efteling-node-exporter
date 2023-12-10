import { fetchAll } from "./efteling-fetcher";
import { convertResponseToMetrics } from "./metric-generator";
import { Registry } from "prom-client";

const registry = new Registry();

console.log("Starting server...");

Bun.serve({
	port: 1337,
	async fetch(req) {
		const url = new URL(req.url);

		if (url.pathname === "/metrics") {
			convertResponseToMetrics(registry, await fetchAll());
			return new Response(await registry.metrics(), {
				headers: {
					"Content-Type": registry.contentType,
				},
			});
		}

		return new Response("404", {
			status: 404,
		});
	},
});
