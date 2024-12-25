import type {
  Feature,
  LineString,
  MultiPolygon,
  Point,
  Polygon,
} from "geojson";

// Types for the WASM API output.
// Need to figure out nicer ways of wiring this up, preferably in a way that the Rust compiler guarantees.

export interface RenderNeighbourhoodOutput {
  type: "FeatureCollection";
  features: (
    | Feature<Polygon, { kind: "boundary"; name: string }>
    | Feature<
        LineString,
        {
          kind: "interior_road";
          shortcuts: number;
          direction: "forwards" | "backwards" | "both";
          direction_edited: boolean;
          road: number;
          // TODO Plus all the stuff from Road::to_gj
        }
      >
    | Feature<
        LineString,
        {
          kind: "crosses";
          pct: number;
        }
      >
    | Feature<Point, { kind: "border_intersection" }>
    | Feature<
        MultiPolygon,
        {
          kind: "cell";
          cell_color: "disconnected" | number;
          // Populated by setCellColors, not in the WASM API
          color?: string;
        }
      >
  )[];
  undo_length: number;
  redo_length: number;
  area_km2: number;
}
