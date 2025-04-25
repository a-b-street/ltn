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

export let populationDensityColorScale = commonQuintileColorScale.toReversed();

export let demandColorScale = commonQuintileColorScale.toReversed();

export let areaLimits = [0.0, 0.3, 0.6, 1.0, 1.5, 2.0];
export let areaColorScale = commonQuintileColorScale.toReversed();

export let stats19ColorScale = commonQuintileColorScale.toReversed();

export let poiColorScale = commonQuintileColorScale.toReversed();

export let carOwnershipLimits = [0, 20, 40, 60, 80, 100];
export let carOwnershipColorScale = commonQuintileColorScale;

export let combinedLimits = [0, 1, 2, 3, 4, 5];
export let combinedColorScale = commonQuintileColorScale.toReversed();

export function bucketize(limits: number[]) {
  let buckets = [];
  for (let i = 1; i < limits.length; i++) {
    buckets.push(i);
  }
  return buckets;
}

// Like the background of the bike/walk sign
export const signGreen = "#0C793A";

export const Style = {
  mapFeature: {
    hover: {
      backgroundColor: "rgb(72, 96, 202)",
    },
  },
};
