export let speedColorScale = [
  "#8a9a5b",
  "#ffc300",
  "#cc5500",
  "#c70039",
  "#900c3f",
  "#581845",
];
export let speedLimitsMPH = [20, 30, 40, 50, 60, 70];
export let speedLimitsKMPH = [30, 50, 65, 80, 95, 110];

// Commonly used color ramp from https://www.ons.gov.uk/census/maps/choropleth, dark to light.
let commonQuintileColorScale = [
  "#080C54",
  "#186290",
  "#1F9EB7",
  "#80C6A3",
  "#CDE594",
];

// The percentiles are [1, 100]. Notice parts of the UI reverse these, to treat
// most deprived ownership (low values) as the most important.
export let simdColorScale = commonQuintileColorScale;
export let simdLimits = [0, 20, 40, 60, 80, 100];

export let populationDensityColorScale = commonQuintileColorScale.toReversed();

export let demandColorScale = commonQuintileColorScale.toReversed();

// From page 7 of https://content.tfl.gov.uk/lsp-app-six-b-strategic-neighbourhoods-analysis-v1.pdf, except removing the smallest bucket to make five
export let areaLimits = [0.0, 0.25, 0.5, 1, 1.5, 2.0];
export let areaColorScale = [
  "#F8E4AF",
  "#9ECC4E",
  "#00BB44",
  "#4ACD8B",
  "#B9E9E9",
];

export let stats19ColorScale = commonQuintileColorScale.toReversed();

export let poiColorScale = commonQuintileColorScale.toReversed();

// Notice parts of the UI reverse these, to treat lowest ownership as the most
// important
export let carOwnershipLimits = [0, 20, 40, 60, 80, 100];
export let carOwnershipColorScale = commonQuintileColorScale;

export let combinedLimits = [1, 2, 3, 4, 5];
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
