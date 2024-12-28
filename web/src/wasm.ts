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

// This is a thin TS wrapper around the auto-generated TS API. The TS
// definitions here are trusted blindly, not checked. Little work should happen
// here aside from parsing and making the API nicer for both the Rust and TS
// code. This is also a step towards moving to using web workers.

export class Backend {
  inner: LTN;

  constructor(
    osmInput: Uint8Array,
    boundary: Feature<Polygon>,
    studyAreaName: string | undefined,
  ) {
    this.inner = new LTN(osmInput, boundary, studyAreaName);
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

  renderNeighbourhood(): RenderNeighbourhoodOutput {
    return JSON.parse(this.inner.renderNeighbourhood());
  }

  renderAutoBoundaries(): FeatureCollection {
    return JSON.parse(this.inner.renderAutoBoundaries());
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

  toggleDirection(road: number) {
    this.inner.toggleDirection(road);
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
  ): FeatureCollection {
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

  impactToOneDestination(
    pt: LngLat,
  ): FeatureCollection & { highest_ratio: number } {
    return JSON.parse(this.inner.impactToOneDestination(pt.lng, pt.lat));
  }
}

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

export type AllShortcuts = FeatureCollection<
  LineString,
  { directness: number; length_meters: number }
>;
