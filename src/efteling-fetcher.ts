import { ApiResponse } from "../types/efteling";

export async function fetchAll(): Promise<ApiResponse> {
	const apiUrl = "https://api.efteling.com/app/wis";

	const response = await fetch(apiUrl);

	return await response.json();
}
