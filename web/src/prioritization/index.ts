import { type DataDrivenPropertyValueSpecification } from "maplibre-gl";
import { makeRamp } from "svelte-utils/map";
import {
  areaColorScale,
  areaLimits,
  carOwnershipColorScale,
  carOwnershipLimits,
  combinedColorScale,
  poiColorScale,
  populationDensityColorScale,
  simdColorScale,
  simdLimits,
  stats19ColorScale,
} from "../common/colors";
import type { MetricBuckets } from "../wasm";

export { default as PrioritizationSelect } from "./PrioritizationSelect.svelte";

export type Prioritization =
  | "none"
  | "car_ownership"
  | "population_density"
  | "pois"
  | "simd"
  | "stats19"
  | "combined";

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
    population_density: makeRamp(
      ["/", ["get", "population"], ["get", "area_km2"]],
      metricBuckets.population_density,
      populationDensityColorScale,
    ),
    pois: makeRamp(
      ["/", ["get", "number_pois"], ["get", "area_km2"]],
      metricBuckets.poi_density,
      poiColorScale,
    ),
    simd: makeRamp(["get", "simd"], simdLimits, simdColorScale),
    stats19: makeRamp(
      ["/", ["get", "number_stats19_collisions"], ["get", "area_km2"]],
      metricBuckets.collision_density,
      stats19ColorScale,
    ),
    combined: makeRamp(
      ["get", "combined_score"],
      [1, 2, 3, 4, 5],
      combinedColorScale,
    ),
  }[selectedPrioritization];
}
