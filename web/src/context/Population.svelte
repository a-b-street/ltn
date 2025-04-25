<script lang="ts">
  import {
    FillLayer,
    hoverStateFilter,
    LineLayer,
    VectorTileSource,
  } from "svelte-maplibre";
  import { makeRamp, Popup } from "svelte-utils/map";
  import {
    ContextLayerButton,
    layerId,
    prettyPrintPercent,
    SequentialLegend,
  } from "../common";
  import {
    bucketize,
    carOwnershipColorScale,
    carOwnershipLimits,
    populationDensityColorScale,
    simdColorScale,
    simdLimits,
  } from "../common/colors";
  import { assetUrl, metricBuckets } from "../stores";

  let showSIMD = false;
  let showPopulationDensity = false;
  let showCarOwnership = false;
</script>

<ContextLayerButton
  label="SIMD"
  bind:show={showSIMD}
  onChange={() => {
    if (showSIMD) {
      showPopulationDensity = false;
      showCarOwnership = false;
    }
  }}
>
  <div slot="legend">
    <SequentialLegend
      colorScale={simdColorScale}
      labels={{ buckets: bucketize(simdLimits) }}
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

<ContextLayerButton
  label="Population density"
  bind:show={showPopulationDensity}
  onChange={() => {
    if (showPopulationDensity) {
      showSIMD = false;
      showCarOwnership = false;
    }
  }}
>
  <div slot="legend">
    <SequentialLegend
      colorScale={populationDensityColorScale}
      labels={{ limits: $metricBuckets.population_density }}
    />
    <div style="display: flex; justify-content: space-between;">
      <span>Less dense</span>
      <span>people / km²</span>
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

<ContextLayerButton
  label="Car ownership"
  bind:show={showCarOwnership}
  onChange={() => {
    if (showCarOwnership) {
      showSIMD = false;
      showPopulationDensity = false;
    }
  }}
>
  <div slot="legend">
    <SequentialLegend
      colorScale={carOwnershipColorScale}
      labels={{ limits: carOwnershipLimits.map((number) => `${number}%`) }}
    />
    <div style="display: flex; justify-content: space-between;">
      <span style="text-align: center; width: 100%">
        Households with at least one car or van
      </span>
    </div>
  </div>

  <p slot="help">
    Show households from the Scottish census with at least one car.
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
    {...layerId("context-population-car-ownership")}
    sourceLayer="population"
    manageHoverState
    paint={{
      "fill-color": makeRamp(
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
      "fill-opacity": hoverStateFilter(0.7, 0.9),
    }}
    layout={{
      visibility: showCarOwnership ? "visible" : "none",
    }}
  >
    <Popup openOn="hover" let:props>
      <p>
        In data zone {props.id}
        {prettyPrintPercent(
          props.households_with_cars_or_vans,
          props.total_households,
        )} of approximately {props.total_households.toLocaleString()}
        households have at least one car or van.
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
        $metricBuckets.population_density,
        populationDensityColorScale,
      ),
      "fill-opacity": hoverStateFilter(0.7, 0.9),
    }}
    layout={{
      visibility: showPopulationDensity ? "visible" : "none",
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
        showSIMD || showPopulationDensity || showCarOwnership
          ? "visible"
          : "none",
    }}
  />
</VectorTileSource>
