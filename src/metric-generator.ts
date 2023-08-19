import { ApiResponse } from "../types/efteling";
import PromClient, { Gauge, Registry } from "prom-client";

const gauges = new Map<string, Gauge>();

export function convertResponseToMetrics(registry: Registry, response: ApiResponse) {
	waitingTimeMetric(registry, response);
}

function waitingTimeMetric(registry: Registry, response: ApiResponse) {
	const waitingTime = response.AttractionInfo.filter(attraction => typeof attraction.WaitingTime !== "undefined").map(
		attraction => ({
			...attraction,
			WaitingTime: attraction.WaitingTime as number,
		}),
	);

	if (!gauges.get("efteling_waiting_time")) {
		let newGauge;
		gauges.set(
			"efteling_waiting_time",
			(newGauge = new PromClient.Gauge({
				name: "efteling_waiting_time",
				help: "Waiting time for attractions",
				labelNames: ["name", "empire", "type"],
			})),
		);

		registry.registerMetric(newGauge);
	}

	const gauge = gauges.get("efteling_waiting_time");

	waitingTime.forEach(attraction => {
		console.log("Setting gauge", attraction.Name, attraction.WaitingTime);
		gauge?.set({ name: attraction.Name, empire: attraction.Empire, type: attraction.Type }, attraction.WaitingTime);
	});
}
