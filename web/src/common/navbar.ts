import type { Mode } from "../stores";

export function pageTitle(mode: Mode["mode"]): string {
  switch (mode) {
    case "title":
      return "Choose project";
    case "new-project":
      return "New project";
    case "pick-neighbourhood":
      return "Pick neighbourhood";
    case "set-boundary":
      return "Adjust boundary";
    case "add-neighbourhood":
      return "Add a neighbourhood";
    case "neighbourhood":
      return "Editing";
    case "view-shortcuts":
      return "View shortcuts";
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
    case "debug-neighbourhood":
      return "Debug neighbourhood";
    case "debug-demand":
      return "Explore origin/destination demand data";
  }
}
