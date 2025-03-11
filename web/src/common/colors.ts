export let speedColorScale = [
  "#8a9a5b",
  "#ffc300",
  "#cc5500",
  "#c70039",
  "#900c3f",
  "#581845",
];
export let speedLimits = [20, 30, 40, 50, 60, 70];

// Commonly used color ramp from https://www.ons.gov.uk/census/maps/choropleth, dark to light.
let commonQuintileColorScale = [
  "#080C54",
  "#186290",
  "#1F9EB7",
  "#80C6A3",
  "#CDE594",
];

export let simdColorScale = commonQuintileColorScale;
// The percentiles are [1, 100]. The 5 colors cover 4 each.
export let simdLimits = [0, 20, 40, 60, 80, 100];

export let densityColorScale = commonQuintileColorScale.toReversed();

// To get raw quintiles, run:
//
//     cd data_prep/scotland && cargo run --bin density_buckets
//     > Raw limits for Scotland density (/ km²): [0, 1324, 2940, 4247, 5858, 52389]
//
// Slightly round those raw numbers:
export let densityLimits = [0, 1_300, 3_000, 4_200, 5_900, 52_000];

export let demandColorScale = commonQuintileColorScale.toReversed();

export let areaLimits = [0.0, 0.3, 0.6, 1.0, 1.5, 2.0];
export let areaColorScale = commonQuintileColorScale.toReversed();

export let stats19Limits = [0, 1.0, 10.0, 50.0, 100.0, 1000.0];
export let stats19ColorScale = commonQuintileColorScale.toReversed();

export let poiLimits = [0, 1.0, 10.0, 50.0, 100.0, 1000.0];
export let poiColorScale = commonQuintileColorScale.toReversed();

export function bucketize(limits: number[]) {
  let buckets = [];
  for (let i = 1; i < limits.length; i++) {
    buckets.push(i);
  }
  return buckets;
}

export const Style = {
  mapFeature: {
    hover: {
      backgroundColor: "blue",
    },
  },
};
