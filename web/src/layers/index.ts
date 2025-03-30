import { type DataDrivenPropertyValueSpecification } from "maplibre-gl";

export { default as CellLayer } from "./CellLayer.svelte";
export { default as HighlightBoundaryLayer } from "./HighlightBoundaryLayer.svelte";
export { default as NeighbourhoodRoadLayer } from "./NeighbourhoodRoadLayer.svelte";
export { default as ModalFilterLayer } from "./ModalFilterLayer.svelte";
export { default as OneWayLayer } from "./OneWayLayer.svelte";
export { default as RenderNeighbourhood } from "./RenderNeighbourhood.svelte";

export function colorByCellColor(): DataDrivenPropertyValueSpecification<string> {
  return [
    "match",
    ["get", "cell_color"],
    "disconnected",
    "red",
    // For numeric values, need to use a step function with % operation
    [
      "let",
      "index",
      ["%", ["to-number", ["get", "cell_color"]], 10],
      [
        "match",
        ["var", "index"],
        0,
        "#8dd3c7",
        1,
        "#ffffb3",
        2,
        "#bebada",
        3,
        "#80b1d3",
        4,
        "#fdb462",
        5,
        "#b3de69",
        6,
        "#fccde5",
        7,
        "#bc80bd",
        8,
        "#ccebc5",
        9,
        "#ffed6f",
        "blue", // fallback is required, but I don't think it's reachable in this case.
      ],
    ],
  ];
}
