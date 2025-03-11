<script lang="ts">
  import {
    FillLayer,
    hoverStateFilter,
    LineLayer,
    VectorTileSource,
  } from "svelte-maplibre";
  import { SequentialLegend } from "svelte-utils";
  import { makeRamp, Popup } from "svelte-utils/map";
  import {
    ContextLayerButton,
    layerId,
    SequentialLegendBucketed,
  } from "../common";
  import {
    bucketize,
    densityColorScale,
    densityLimits,
    simdColorScale,
    simdLimits,
  } from "../common/colors";
  import { assetUrl } from "../stores";

  let showSIMD = false;
  let showDensity = false;
</script>

<ContextLayerButton label="SIMD" bind:show={showSIMD}>
  <div slot="legend">
    <SequentialLegendBucketed
      colorScale={simdColorScale}
      buckets={bucketize(simdLimits)}
    />
    <div style="display: flex; justify-content: space-between;">
      <span>More deprived</span>
      <span>Less deprived</span>
    </div>
  </div>

  <p slot="help">
    This shows the Scottish Index of Multiple Deprivation (SIMD) from <a
      href="https://www.data.gov.uk/dataset/1102bf85-ed49-440a-b211-da87e8d752eb/scottish-index-of-multiple-deprivation-simd-2020"
      target="_blank"
    >
      2020 data
    </a>
    . SIMD combines different domains: income; employment; health; education, skills
    and training; geographic access to services; crime; and housing.
  </p>
</ContextLayerButton>

<ContextLayerButton label="Population density" bind:show={showDensity}>
  <div slot="legend">
    <SequentialLegend colorScale={densityColorScale} limits={densityLimits} />
    <div style="display: flex; justify-content: space-between;">
      <span>Less less</span>
      <span>More dense</span>
    </div>
  </div>

  <p slot="help">
    This shows population data from <a
      href="https://www.data.gov.uk/dataset/1102bf85-ed49-440a-b211-da87e8d752eb/scottish-index-of-multiple-deprivation-simd-2020"
      target="_blank"
    >
      2020 data
    </a>
    .
  </p>
</ContextLayerButton>

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
      visibility: showSIMD ? "visible" : "none",
    }}
  >
    <Popup openOn="hover" let:props>
      <p>
        Data zone {props.id}
        has {props.population.toLocaleString()}
        people, and a SIMD rank of {props.imd_rank}, making it less deprived
        than {props.imd_percentile}% of data zones.
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
      visibility: showSIMD || showDensity ? "visible" : "none",
    }}
  />
</VectorTileSource>
