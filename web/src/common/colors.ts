export let speedColorScale = [
  "#8a9a5b",
  "#ffc300",
  "#cc5500",
  "#c70039",
  "#900c3f",
  "#581845",
];
export let speedLimits = [20, 30, 40, 50, 60, 70];

// Color ramp from https://www.ons.gov.uk/census/maps/choropleth, dark to light.
export let simdColorScale = [
  "#080C54",
  "#186290",
  "#1F9EB7",
  "#80C6A3",
  "#CDE594",
];

// The percentiles are [1, 100]. The 5 colors cover 4 each.
export let simdLimits = [0, 20, 40, 60, 80, 100];

export const Style = {
  mapFeature: {
    hover: {
      backgroundColor: "blue",
    },
  },
};

// From https://www.ons.gov.uk/census/maps/choropleth
export let demandColorScale = [
  "#CDE594",
  "#80C6A3",
  "#1F9EB7",
  "#186290",
  "#080C54",
];

export let areaLimits = [0.0, 0.3, 0.6, 1.0, 1.5, 2.0];

export let stats19Limits = [0, 1.0, 10.0, 50.0, 100.0, 1000.0];
export let stats19ColorScale = demandColorScale;
