import { ApiResponse, Type } from "./types/efteling";
import PromClient, { Gauge, Registry } from "prom-client";

const gauges = new Map<string, Gauge>();

export function convertResponseToMetrics(registry: Registry, response: ApiResponse) {
	waitingTimeMetric(registry, response);
}

function waitingTimeMetric(registry: Registry, response: ApiResponse) {
	const waitingTime = response.AttractionInfo.filter(attr => attr.Type === Type.Attracties).map(attraction => ({
		...attraction,
		WaitingTime: attraction.WaitingTime as number,
	}));

	if (!gauges.get("efteling_waiting_time")) {
		let newGauge;
		gauges.set(
			"efteling_waiting_time",
			(newGauge = new PromClient.Gauge({
				name: "efteling_waiting_time",
				help: "Waiting time for attractions",
				labelNames: ["id", "name", "empire", "type"],
			})),
		);

		registry.registerMetric(newGauge);
	}

	const gauge = gauges.get("efteling_waiting_time");

	waitingTime.forEach(attraction => {
		if (typeof attraction.WaitingTime !== "undefined") {
			console.log("Setting gauge", attraction.Name, attraction.WaitingTime);
			gauge?.set(
				{ id: attraction.Id, name: attraction.Name, empire: attraction.Empire, type: attraction.Type },
				attraction.WaitingTime,
			);
		} else {
			gauge?.remove({ id: attraction.Id, name: attraction.Name, empire: attraction.Empire, type: attraction.Type });
		}
	});
}
