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
import * as backendPkg from "../../backend/pkg";
import { sum } from "./common";
import type { Intersection, IntersectionFeature } from "./common/Intersection";
import type {
  ProjectFeatureCollection,
  StudyAreaName,
} from "./common/ProjectStorage";
import type { AppFocus } from "./stores";

// This is a thin TS wrapper around the auto-generated TS API. The TS
// definitions here are trusted blindly, not checked. Little work should happen
// here aside from parsing and making the API nicer for both the Rust and TS
// code. This is also a step towards moving to using web workers.

export type NeighbourhoodDefinitionFeature = Feature<
  Polygon,
  {
    name: string;
    waypoints?: Waypoint[];
    kind: "boundary";
  }
>;

export type NeighbourhoodBoundaryFeature = Feature<
  Polygon,
  {
    name: string;
    waypoints: Waypoint[];
  } & GeneratedBoundaryFeature["properties"]
>;

export type GeneratedBoundaryFeature = Feature<
  Polygon,
  {
    area_km2: number;
    households_with_cars_or_vans: number;
    total_households: number;
    population: number;
    simd: number;
    number_stats19_collisions: number;
    number_pois: number;
    combined_score: number;
  }
>;

export class Backend {
  inner: backendPkg.LTN;

  constructor(
    mapModelInput: Uint8Array | undefined,
    osmInput: Uint8Array | undefined,
    boundary: Feature<Polygon | MultiPolygon> | undefined,
    appFocus: AppFocus,
    studyAreaName: StudyAreaName,
    projectName: string,
    dbSchemaVersion: number,
  ) {
    this.inner = new backendPkg.LTN(
      mapModelInput || new Uint8Array(),
      osmInput || new Uint8Array(),
      boundary,
      appFocus,
      studyAreaName,
      projectName,
      dbSchemaVersion,
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

  renderModalFiltersBeforeEdits(): FeatureCollection {
    return JSON.parse(this.inner.renderModalFiltersBeforeEdits());
  }

  renderTurnRestrictions(): TurnRestrictions {
    return JSON.parse(this.inner.renderTurnRestrictions());
  }

  renderTurnRestrictionsBeforeEdits(): TurnRestrictions {
    return JSON.parse(this.inner.renderTurnRestrictionsBeforeEdits());
  }

  // This adds a 'color' property to all cells. It's nicer to keep this on the
  // frontend, since it's about styling.
  renderNeighbourhood(): RenderNeighbourhoodOutput {
    let gj: RenderNeighbourhoodOutput = JSON.parse(
      this.inner.renderNeighbourhood(),
    );
    gj.maxShortcuts =
      Math.max(
        ...gj.features.map((f) =>
          f.properties.kind == "interior_road" ? f.properties.shortcuts : 0,
        ),
      ) ?? 0;
    return gj;
  }

  // This adds a 'color' property to all cells. It's nicer to keep this on the
  // frontend, since it's about styling.
  renderNeighbourhoodBeforeEdits(): RenderNeighbourhoodOutput {
    let gj: RenderNeighbourhoodOutput = JSON.parse(
      this.inner.renderNeighbourhoodBeforeEdits(),
    );
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
    toMerge: FeatureCollection<Polygon>,
  ): GeneratedBoundaryFeature {
    let serializedMergedBoundary = this.inner.generateMergedBoundary(toMerge);
    return JSON.parse(serializedMergedBoundary);
  }

  setCurrentNeighbourhoodBoundary(name: string, input: Feature) {
    this.inner.setCurrentNeighbourhoodBoundary(name, input);
  }

  extractWaypointsFromRing(line_string: LineString): Waypoint[] {
    let serializedWaypoints = this.inner.extractWaypointsFromRing(line_string);
    return JSON.parse(serializedWaypoints);
  }

  deleteNeighbourhoodBoundary(name: string) {
    this.inner.deleteNeighbourhoodBoundary(name);
  }

  renameNeighbourhoodBoundary(oldName: string, newName: string) {
    this.inner.renameNeighbourhoodBoundary(oldName, newName);
  }

  setCurrentNeighbourhood(name: string) {
    this.inner.setCurrentNeighbourhood(name);
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

  toggleMainRoad(road: number) {
    this.inner.toggleMainRoad(road);
  }

  setMainRoads(intersections: number[], makeMainRoad: boolean) {
    this.inner.setMainRoads(new Uint32Array(intersections), makeMainRoad);
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

  getAllShortcutsBeforeEdits(): AllShortcuts {
    return JSON.parse(this.inner.getAllShortcutsBeforeEdits());
  }

  toSavefile(): ProjectFeatureCollection {
    return JSON.parse(this.inner.toSavefile());
  }

  loadSavefile(gj: ProjectFeatureCollection) {
    this.inner.loadSavefile(gj);
  }

  changeProjectName(name: string) {
    this.inner.changeProjectName(name);
  }

  compareRoute(
    pt1: LngLat,
    pt2: LngLat,
    mainRoadPenalty: number,
  ): CompareRoute {
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

  predictImpact(fastSample: boolean): Impact {
    return JSON.parse(this.inner.predictImpact(fastSample));
  }

  getImpactsOnRoad(road: number, fastSample: boolean): ImpactOnRoad[] {
    return JSON.parse(this.inner.getImpactsOnRoad(road, fastSample)).map(
      (x: any) => {
        let [count, before, after] = x;
        return { count, before, after };
      },
    );
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

  getMetricBuckets(): MetricBuckets | null {
    try {
      return JSON.parse(this.inner.getMetricBuckets());
    } catch (err) {
      // This area doesn't have any
      return null;
    }
  }

  eraseAllMainRoads() {
    this.inner.eraseAllMainRoads();
  }

  getOsmTimestamp(): Date | null {
    let t = this.inner.getOsmTimestamp();
    if (t) {
      return new Date(1000 * Number(t));
    }
    return null;
  }
}

export type Impact = FeatureCollection<
  LineString,
  { id: number; before: number; after: number }
> & { max_count: number };

export interface ImpactOnRoad {
  count: number;
  before: Feature<LineString, { kind: "before" }> | null;
  after: Feature<LineString, { kind: "after" }> | null;
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
          road_kind: "private" | "pedestrian" | "service" | "normal";
          shortcuts: number;
          travel_flow: "forwards" | "backwards" | "both";
          travel_flow_edited: boolean;
          edited: boolean;
          road: number;
          cell_color: "disconnected" | "pedestrianized" | number;
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
        Point,
        {
          kind: "border_entry";
          cell_color: "disconnected" | "pedestrianized" | number;
          bearing_upon_entry: number;
          major_junction: boolean;
        }
      >
    | Feature<
        MultiPolygon,
        {
          kind: "cell";
          cell_color: "disconnected" | "pedestrianized" | number;
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

export type CompareRoute = FeatureCollection<
  LineString,
  { kind: "before" | "after"; distance: number; time: number }
>;

export interface MetricBuckets {
  population_density: number[];
  collision_density: number[];
  poi_density: number[];
}

export type TurnRestrictions = FeatureCollection<
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
>;
