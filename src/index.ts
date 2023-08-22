import express from "express";
import { fetchAll } from "./efteling-fetcher";
import { convertResponseToMetrics } from "./metric-generator";
import { Registry } from "prom-client";

const registry = new Registry();

const app = express();

app.get("/metrics", async (req, res) => {
	try {
		convertResponseToMetrics(registry, await fetchAll());

		res.set("Content-Type", registry.contentType);
		res.end(await registry.metrics());
	} catch (e) {
		console.error(e);
		res.sendStatus(500);
	}
});

app.listen(1337, () => {
	console.log(`⚡️[server]: Server is running at http://localhost:1337`);
});
