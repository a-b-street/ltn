import { get } from "svelte/store";
import { currentNeighbourhoodName, currentProject, type Mode } from "../stores";

export function pageTitle(mode: Mode["mode"]): string {
  switch (mode) {
    case "title":
      return "Choose project";
    case "new-project":
      return "New project";
    case "pick-neighbourhood":
      return get(currentProject)?.project_name || "Pick neighbourhood"; // TODO truncate if necessary
    case "set-boundary":
      return "Adjust boundary";
    case "add-neighbourhood":
      return "Add a neighbourhood";
    case "neighbourhood":
      return get(currentNeighbourhoodName) || "Editing"; // TODO truncate if necessary
    case "view-shortcuts":
      return "Shortcuts";
    case "impact-one-destination":
      return "Impact on one destination";
    case "route":
      return "Route";
    case "predict-impact":
      return "Predict impact";
    case "impact-detail":
      return "Impact detail";
    case "debug-intersections":
      return "Debug intersections";
    case "debug-traffic":
      return "Debug traffic";
    case "debug-neighbourhood":
      return "Debug neighbourhood";
    case "debug-demand":
      return "Explore origin/destination demand data";
  }
}
