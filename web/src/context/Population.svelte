<script lang="ts">
  import { onMount } from "svelte";
  import {
    FillLayer,
    GeoJSON,
    hoverStateFilter,
    LineLayer,
    VectorTileSource,
  } from "svelte-maplibre";
  import { SequentialLegend } from "svelte-utils";
  import { emptyGeojson, makeRamp, Popup } from "svelte-utils/map";
  import { HelpButton, layerId } from "../common";
  import { simdColorScale, simdLimits } from "../common/colors";
  import { assetUrl, backend } from "../stores";

  let showSIMD = false;
  let aggregateSIMDToNeighbourhoods = false;
  let showDensity = false;

  $: neighbourhoods = emptyGeojson();
  onMount(() => {
    if ($backend) {
      neighbourhoods = $backend!.getAllNeighbourhoods();
    } else {
      backend.subscribe((b) => {
        if (b) {
          console.log("backend loaded async");
          neighbourhoods = b.getAllNeighbourhoods();
        } else {
          console.log("backend still not loaded");
        }
      });
    }
  });
  let densityColorScale = simdColorScale.toReversed();
  // Use the same (slightly rounded) buckets as https://www.ons.gov.uk/census/maps/choropleth/population/population-density/population-density/persons-per-square-kilometre. TODO Adapt for Scotland.
  let densityLimits = [0, 4700, 13000, 33000, 94000, 1980000];
</script>

<button class="secondary" on:click={() => (showSIMD = !showSIMD)}>SIMD</button>
{#if showSIMD}
  <SequentialLegend colorScale={simdColorScale} limits={simdLimits} />
  <p>Darker colours are more deprived</p>
  <label>
    <input type="checkbox" bind:checked={aggregateSIMDToNeighbourhoods} />
    Aggregate to LTN boundaries.
  </label>
  <HelpButton>
    <p>
      This shows the Scottish Index of Multiple Deprivation (SIMD) from <a
        href="https://www.data.gov.uk/dataset/1102bf85-ed49-440a-b211-da87e8d752eb/scottish-index-of-multiple-deprivation-simd-2020"
        target="_blank"
      >
        2020 data
      </a>
      . SIMD combines different domains: income; employment; health; education, skills
      and training; geographic access to services; crime; and housing.
    </p>
  </HelpButton>
{/if}

<button class="secondary" on:click={() => (showDensity = !showDensity)}>
  Population density
</button>
{#if showDensity}
  <SequentialLegend colorScale={densityColorScale} limits={densityLimits} />
  <p>Darker colours are denser</p>

  <HelpButton>
    <p>
      This shows population data from <a
        href="https://www.data.gov.uk/dataset/1102bf85-ed49-440a-b211-da87e8d752eb/scottish-index-of-multiple-deprivation-simd-2020"
        target="_blank"
      >
        2020 data
      </a>
      .
    </p>
  </HelpButton>
{/if}

<GeoJSON data={neighbourhoods} generateId>
  <FillLayer
    {...layerId("context-simd-per-neighbourhood")}
    manageHoverState
    paint={{
      "fill-color": makeRamp(["get", "simd"], simdLimits, simdColorScale),
      "fill-opacity": hoverStateFilter(0.7, 0.9),
    }}
    layout={{
      visibility:
        showSIMD && aggregateSIMDToNeighbourhoods ? "visible" : "none",
    }}
  >
    <!-- REVIEW: handle click to take you to neighbourhood? -->
    <Popup openOn="hover" let:props>
      <h2>{props.name}</h2>
      <b>SIMD:</b>
      {props.simd.toFixed(1)}
    </Popup>
  </FillLayer>
</GeoJSON>

<VectorTileSource
  url={`pmtiles://${assetUrl("cnt_layers/population.pmtiles")}`}
>
  <FillLayer
    {...layerId("context-simd")}
    sourceLayer="population"
    manageHoverState
    paint={{
      "fill-color": makeRamp(
        ["get", "imd_percentile"],
        simdLimits,
        simdColorScale,
      ),
      "fill-opacity": hoverStateFilter(0.7, 0.9),
    }}
    layout={{
      visibility:
        showSIMD && !aggregateSIMDToNeighbourhoods ? "visible" : "none",
    }}
  >
    <Popup openOn="hover" let:props>
      <p>
        Data zone {props.id}
        has {props.population.toLocaleString()}
        people, and a SIMD rank of {props.imd_rank}, putting it in the {props.imd_percentile}
        percentile.
      </p>
    </Popup>
  </FillLayer>

  <FillLayer
    {...layerId("context-population-density")}
    sourceLayer="population"
    manageHoverState
    paint={{
      "fill-color": makeRamp(
        ["/", ["get", "population"], ["/", ["get", "area"], 1e6]],
        densityLimits,
        densityColorScale,
      ),
      "fill-opacity": hoverStateFilter(0.7, 0.9),
    }}
    layout={{
      visibility: showDensity ? "visible" : "none",
    }}
  >
    <Popup openOn="hover" let:props>
      <p>
        Data zone {props.id}
        has {props.population.toLocaleString()}
        people, with a density of {Math.round(
          props.population / (props.area / 1e6),
        ).toLocaleString()} people per square kilometer
      </p>
    </Popup>
  </FillLayer>

  <LineLayer
    {...layerId("context-population-outline")}
    sourceLayer="population"
    paint={{ "line-color": "black", "line-width": 1 }}
    layout={{
      visibility:
        (showSIMD && !aggregateSIMDToNeighbourhoods) || showDensity
          ? "visible"
          : "none",
    }}
  />
</VectorTileSource>
