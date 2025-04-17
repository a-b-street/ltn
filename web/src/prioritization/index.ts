import { type DataDrivenPropertyValueSpecification } from "maplibre-gl";
import { makeRamp } from "svelte-utils/map";
import {
  areaColorScale,
  areaLimits,
  carOwnershipColorScale,
  carOwnershipLimits,
  densityColorScale,
  poiColorScale,
  poiLimits,
  simdColorScale,
  simdLimits,
  stats19ColorScale,
  stats19Limits,
} from "../common/colors";
import type { MetricBuckets } from "../wasm";

export { default as PrioritizationSelect } from "./PrioritizationSelect.svelte";

export type Prioritization =
  | "none"
  | "car_ownership"
  | "density"
  | "pois"
  | "simd"
  | "stats19";

export function prioritizationFillColor(
  noneColor: { none: DataDrivenPropertyValueSpecification<string> },
  selectedPrioritization: Prioritization,
  metricBuckets: MetricBuckets,
): DataDrivenPropertyValueSpecification<string> {
  return {
    none: noneColor.none,
    area: makeRamp(["get", "area_km2"], areaLimits, areaColorScale),
    car_ownership: makeRamp(
      [
        "*",
        100,
        [
          "/",
          ["get", "households_with_cars_or_vans"],
          ["get", "total_households"],
        ],
      ],
      carOwnershipLimits,
      carOwnershipColorScale,
    ),
    density: makeRamp(
      ["/", ["get", "population"], ["get", "area_km2"]],
      metricBuckets.population_density,
      densityColorScale,
    ),
    pois: makeRamp(
      ["/", ["get", "number_pois"], ["get", "area_km2"]],
      poiLimits,
      poiColorScale,
    ),
    simd: makeRamp(["get", "simd"], simdLimits, simdColorScale),
    stats19: makeRamp(
      ["/", ["get", "number_stats19_collisions"], ["get", "area_km2"]],
      stats19Limits,
      stats19ColorScale,
    ),
  }[selectedPrioritization];
}
