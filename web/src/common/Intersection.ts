import type { Feature, Point } from "geojson";

export type IntersectionId = number;

export type DiagonalFilter = {
  is_rotated: boolean;
  angle: number;
};

export type IntersectionProperties = {
  kind: "editable_intersection";
  intersection_id: number;
  filter?: DiagonalFilter;
};
export type IntersectionFeature = Feature<Point, IntersectionProperties>;

export class Intersection {
  feature: IntersectionFeature;
  constructor(feature: IntersectionFeature) {
    this.feature = feature;
  }

  get intersectionId(): IntersectionId {
    return this.feature.properties.intersection_id;
  }

  get filter(): DiagonalFilter | undefined {
    return this.feature.properties.filter;
  }

  get filterAngle(): number | undefined {
    return this.feature.properties.filter?.angle;
  }

  get hasRotatedFilter(): boolean {
    return this.feature.properties.filter?.is_rotated ?? false;
  }
}
