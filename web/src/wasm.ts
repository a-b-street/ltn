import { LTN } from "backend";
import type {
  Feature,
  FeatureCollection,
  LineString,
  MultiPolygon,
  Point,
  Polygon,
} from "geojson";
import type { LngLat } from "maplibre-gl";
import type { Waypoint } from "route-snapper-ts";
import { sum } from "./common";
import type { Intersection, IntersectionFeature } from "./common/Intersection";

// This is a thin TS wrapper around the auto-generated TS API. The TS
// definitions here are trusted blindly, not checked. Little work should happen
// here aside from parsing and making the API nicer for both the Rust and TS
// code. This is also a step towards moving to using web workers.

export type NeighbourhoodBoundaryFeature = Feature<
  Polygon,
  {
    name: string;
    waypoints: Waypoint[];

    area_km2: number;
    population: number;
    simd: number;
    number_stats19_collisions: number;
    number_pois: number;
  }
>;

export type GeneratedBoundaryFeature = Feature<
  Polygon,
  {
    touches_big_road: boolean;
    touches_railway: boolean;
    touches_waterway: boolean;

    area_km2: number;
    population: number;
    simd: number;
    number_stats19_collisions: number;
    number_pois: number;
  }
>;

export class Backend {
  inner: LTN;

  constructor(
    osmInput: Uint8Array,
    demandInput: Uint8Array | undefined,
    contextDataInput: Uint8Array | undefined,
    boundary: Feature<Polygon>,
    studyAreaName: string | undefined,
  ) {
    this.inner = new LTN(
      osmInput,
      demandInput || new Uint8Array(),
      contextDataInput || new Uint8Array(),
      boundary,
      studyAreaName,
    );
  }

  getInvertedBoundary(): Feature<Polygon> {
    return JSON.parse(this.inner.getInvertedBoundary());
  }

  getBounds(): [number, number, number, number] {
    return Array.from(this.inner.getBounds()) as [
      number,
      number,
      number,
      number,
    ];
  }

  toRouteSnapper(): Uint8Array {
    return this.inner.toRouteSnapper();
  }

  // We could parse and express the GJ types here, but the only use is currently just to dump for debugging
  toRouteSnapperGj(): string {
    return this.inner.toRouteSnapperGj();
  }

  // TODO Improve types below
  renderModalFilters(): FeatureCollection {
    return JSON.parse(this.inner.renderModalFilters());
  }

  renderTurnRestrictions(): FeatureCollection<
    Point,
    {
      kind: TurnRestrictionKind;
      icon_angle: number;
      // GeoJSON geometries stringified
      from_geometry: string;
      to_geometry: string;
      edited: boolean;
      intersection: number;
      from_road: number;
      to_road: number;
    }
  > {
    return JSON.parse(this.inner.renderTurnRestrictions());
  }

  // This adds a 'color' property to all cells. It's nicer to keep this on the
  // frontend, since it's about styling.
  renderNeighbourhood(): RenderNeighbourhoodOutput {
    let gj = setCellColors(JSON.parse(this.inner.renderNeighbourhood()));
    gj.maxShortcuts =
      Math.max(
        ...gj.features.map((f) =>
          f.properties.kind == "interior_road" ? f.properties.shortcuts : 0,
        ),
      ) ?? 0;
    return gj;
  }

  generatedBoundaries(): FeatureCollection<
    GeneratedBoundaryFeature["geometry"],
    GeneratedBoundaryFeature["properties"]
  > {
    return JSON.parse(this.inner.generatedBoundaries());
  }

  generateMergedBoundary(
    toMerge: FeatureCollection<
      GeneratedBoundaryFeature["geometry"],
      GeneratedBoundaryFeature["properties"]
    >,
  ): GeneratedBoundaryFeature {
    let serializedMergedBoundary = this.inner.generateMergedBoundary(toMerge);
    return JSON.parse(serializedMergedBoundary);
  }

  setNeighbourhoodBoundary(name: string, input: Feature) {
    this.inner.setNeighbourhoodBoundary(name, input);
  }

  deleteNeighbourhoodBoundary(name: string) {
    this.inner.deleteNeighbourhoodBoundary(name);
  }

  renameNeighbourhoodBoundary(oldName: string, newName: string) {
    this.inner.renameNeighbourhoodBoundary(oldName, newName);
  }

  setCurrentNeighbourhood(name: string, editPerimeterRoads: boolean) {
    this.inner.setCurrentNeighbourhood(name, editPerimeterRoads);
  }

  addModalFilter(pt: LngLat, kind: string) {
    this.inner.addModalFilter(pt, kind);
  }

  addManyModalFilters(line: Feature<LineString>, kind: string) {
    this.inner.addManyModalFilters(line, kind);
  }

  deleteModalFilter(road: number) {
    this.inner.deleteModalFilter(road);
  }

  addDiagonalFilter(intersection: Intersection) {
    this.inner.addDiagonalFilter(intersection.intersectionId);
  }

  rotateDiagonalFilter(intersection: Intersection) {
    this.inner.rotateDiagonalFilter(intersection.intersectionId);
  }

  deleteDiagonalFilter(intersection: Intersection) {
    this.inner.deleteDiagonalFilter(intersection.intersectionId);
  }

  toggleTravelFlow(road: number) {
    this.inner.toggleTravelFlow(road);
  }

  addTurnRestriction(from_road: number, to_road: number) {
    this.inner.addTurnRestriction(from_road, to_road);
  }

  deleteTurnRestriction(
    intersection: number,
    from_road: number,
    to_road: number,
  ) {
    this.inner.deleteTurnRestriction(intersection, from_road, to_road);
  }

  getTurnRestrictionTargets(
    road: number,
  ): FeatureCollection<
    LineString,
    { road: number; name: string; kind: TurnRestrictionKind }
  > {
    return JSON.parse(this.inner.getTurnRestrictionTargets(road));
  }

  undo() {
    this.inner.undo();
  }

  redo() {
    this.inner.redo();
  }

  getShortcutsCrossingRoad(road: number): AllShortcuts {
    return JSON.parse(this.inner.getShortcutsCrossingRoad(road));
  }

  getAllShortcuts(): AllShortcuts {
    return JSON.parse(this.inner.getAllShortcuts());
  }

  toSavefile(): FeatureCollection {
    return JSON.parse(this.inner.toSavefile());
  }

  loadSavefile(gj: FeatureCollection) {
    this.inner.loadSavefile(gj);
  }

  compareRoute(
    pt1: LngLat,
    pt2: LngLat,
    mainRoadPenalty: number,
  ): FeatureCollection<
    LineString,
    { kind: "before" | "after"; distance: number; time: number }
  > {
    return JSON.parse(
      this.inner.compareRoute(
        pt1.lng,
        pt1.lat,
        pt2.lng,
        pt2.lat,
        mainRoadPenalty,
      ),
    );
  }

  impactToOneDestination(pt: LngLat): FeatureCollection<
    LineString,
    {
      distance_before: number;
      distance_after: number;
      time_before: number;
      time_after: number;
      pt1_x: number;
      pt1_y: number;
    }
  > & { highest_time_ratio: number } {
    return JSON.parse(this.inner.impactToOneDestination(pt.lng, pt.lat));
  }

  predictImpact(): FeatureCollection<
    LineString,
    { id: number; before: number; after: number }
  > & { max_count: number } {
    return JSON.parse(this.inner.predictImpact());
  }

  getImpactsOnRoad(
    road: number,
  ): Array<
    [
      Feature<LineString, { kind: "before" }>,
      Feature<LineString, { kind: "after" }>,
    ]
  > {
    return JSON.parse(this.inner.getImpactsOnRoad(road));
  }

  getAllNeighbourhoods(): FeatureCollection<
    NeighbourhoodBoundaryFeature["geometry"],
    NeighbourhoodBoundaryFeature["properties"]
  > {
    return JSON.parse(this.inner.getAllNeighbourhoods());
  }

  getAllIntersections(): FeatureCollection<
    Point,
    { intersection_id: number; has_turn_restrictions: boolean; osm: string }
  > {
    return JSON.parse(this.inner.getAllIntersections());
  }

  getMovements(intersection: number): FeatureCollection<Polygon> {
    return JSON.parse(this.inner.getMovements(intersection));
  }

  getDemandModel(): FeatureCollection<MultiPolygon, ZoneDemandProps> {
    let gj = JSON.parse(this.inner.getDemandModel());
    for (let f of gj.features) {
      f.properties.sum_from = sum(f.properties.counts_from);
      f.properties.sum_to = sum(f.properties.counts_to);
    }
    return gj;
  }

  getPOIs(): FeatureCollection<Point, { name?: string; kind: string }> {
    return JSON.parse(this.inner.getPOIs());
  }
}

type TurnRestrictionKind =
  | "left"
  | "right"
  | "straight"
  | "u_left_to_right"
  | "u_right_to_left";

export type ZoneDemandProps = {
  name: string;
  counts_from: number[];
  counts_to: number[];
  sum_from: number;
  sum_to: number;
};

export interface RenderNeighbourhoodOutput {
  type: "FeatureCollection";
  features: (
    | Feature<Polygon, { kind: "boundary"; name: string }>
    | Feature<
        LineString,
        {
          kind: "interior_road";
          shortcuts: number;
          travel_flow: "forwards" | "backwards" | "both";
          travel_flow_edited: boolean;
          edited: boolean;
          road: number;
          cell_color: "disconnected" | number;
          speed_mph: number;
          // Populated by setCellColors, not in the Rust backend
          color: string;
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
        Polygon,
        {
          kind: "border_arrow";
          cell_color: "disconnected" | number;
          // Populated by setCellColors, not in the Rust backend
          color: string;
        }
      >
    | Feature<
        MultiPolygon,
        {
          kind: "cell";
          cell_color: "disconnected" | number;
          // Populated by setCellColors, not in the Rust backend
          color: string;
        }
      >
    | IntersectionFeature
  )[];
  undo_length: number;
  redo_length: number;
  area_km2: number;
  // Populated by this wrapper, not in the Rust backend
  maxShortcuts: number;
}

export type AllShortcuts = FeatureCollection<
  LineString,
  { directness: number; length_meters: number }
>;

// Sets a 'color' property on any cell polygons. Idempotent.
function setCellColors(
  gj: RenderNeighbourhoodOutput,
): RenderNeighbourhoodOutput {
  // A qualitative palette from colorbrewer2.org, skipping the red hue (used
  // for levels of shortcutting) and grey (too close to the basemap)
  let cell_colors = [
    "#8dd3c7",
    "#ffffb3",
    "#bebada",
    "#80b1d3",
    "#fdb462",
    "#b3de69",
    "#fccde5",
    "#bc80bd",
    "#ccebc5",
    "#ffed6f",
  ];

  for (let f of gj.features) {
    if (
      f.properties.kind != "cell" &&
      f.properties.kind != "border_arrow" &&
      f.properties.kind != "interior_road"
    ) {
      continue;
    }
    if (f.properties.cell_color == "disconnected") {
      f.properties.color = "red";
    } else {
      f.properties.color =
        cell_colors[f.properties.cell_color % cell_colors.length];
    }
  }

  return gj;
}
