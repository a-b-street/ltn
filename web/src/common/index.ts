export { default as BasemapPicker } from "./BasemapPicker.svelte";
export { default as DisableInteractiveLayers } from "./DisableInteractiveLayers.svelte";
export { default as HelpButton } from "./HelpButton.svelte";
export { default as Link } from "./Link.svelte";
export { default as StreetView } from "./StreetView.svelte";
export { layerId } from "./zorder";

// TS fix for the imprecise geojson types
export function gjPosition(pt: number[]): [number, number] {
  return pt as [number, number];
}
