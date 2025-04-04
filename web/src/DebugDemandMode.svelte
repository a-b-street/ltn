<script lang="ts">
  import type { Feature, FeatureCollection, MultiPolygon } from "geojson";
  import type { DataDrivenPropertyValueSpecification } from "maplibre-gl";
  import {
    FillLayer,
    GeoJSON,
    hoverStateFilter,
    LineLayer,
  } from "svelte-maplibre";
  import { emptyGeojson, makeRamp } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import { layerId, ModeLink, pageTitle, SequentialLegend } from "./common";
  import { demandColorScale } from "./common/colors";
  import { backend, mode, zoomToDefault } from "./stores";
  import type { ZoneDemandProps } from "./wasm";

  let gj = emptyGeojson() as FeatureCollection<MultiPolygon, ZoneDemandProps>;
  try {
    gj = $backend!.getDemandModel();
  } catch (err) {
    window.alert("No demand model for this area");
    $mode = { mode: "pick-neighbourhood" };
  }

  let showTo = false;

  let hovered: Feature | null = null;
  $: hoveredId = hovered == null ? null : (hovered.id as number);
  // MapLibre doesn't preserve the arrays in properties, so use the original version
  $: current = hoveredId != null ? gj.features[hoveredId] : null;

  $: [limits, fillColor] = getLimitsAndColor(hoveredId, showTo);

  function getLimitsAndColor(
    hoveredId: number | null,
    showTo: boolean,
  ): [number[], DataDrivenPropertyValueSpecification<string>] {
    if (hoveredId == null) {
      let key: "sum_from" | "sum_to" = showTo ? "sum_to" : "sum_from";

      let max = Math.max(...gj.features.map((f) => f.properties[key]));
      let limits = maxIntoBuckets(max);

      return [limits, makeRamp(["get", key], limits, demandColorScale)];
    } else {
      // The GJ has nested arrays with the counts, but MapLibre stringifies
      // these properties, so we can't use them for styling. Instead we
      // dynamically make an expression to color each neighbourhood, by
      // embedding the counts. The number of zones doesn't get too big,
      // so this works well enough.
      let key: "counts_from" | "counts_to" = showTo
        ? "counts_to"
        : "counts_from";
      let counts = [...gj.features.map((f) => f.properties[key][hoveredId])];

      let max = Math.max(...counts);
      let limits = maxIntoBuckets(max);

      return [
        limits,
        makeRamp(["at", ["id"], ["literal", counts]], limits, demandColorScale),
      ];
    }
  }

  function maxIntoBuckets(max: number): number[] {
    let n = demandColorScale.length + 1;
    return Array.from(Array(n).keys()).map((i) =>
      Math.round((max / (n - 1)) * i),
    );
  }
</script>

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <ModeLink mode={{ mode: "title" }} afterLink={zoomToDefault} />
        </li>
        <li>
          <ModeLink mode={{ mode: "pick-neighbourhood" }} />
        </li>
        <li>
          <ModeLink mode={{ mode: "predict-impact" }} />
        </li>
        <li>{pageTitle($mode.mode)}</li>
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <BackButton mode={{ mode: "predict-impact" }} />

    <!-- TODO Plumb through metadata about the sources used -->

    <p>Trips begin and end in {gj.features.length.toLocaleString()} zones</p>

    <label>
      Trips from
      <input type="checkbox" role="switch" bind:checked={showTo} />
      Trips to
    </label>

    <p>
      Trips {showTo ? "from" : "to"}
      {hoveredId == null ? "each zone" : "this zone"}:
    </p>
    <SequentialLegend colorScale={demandColorScale} labels={{ limits }} />

    <hr />

    {#if current && hoveredId != null}
      <u>{current.properties.name}</u>
      <p>
        Total trips from here: {current.properties.sum_from.toLocaleString()}
      </p>
      <p>
        Total trips to here: {current.properties.sum_to.toLocaleString()}
      </p>
      <p>
        Total intra-zonal trips starting and ending here: {current.properties
          .counts_from[hoveredId]}
      </p>
    {:else}
      <p>Hover on a zone</p>
    {/if}
  </div>

  <div slot="map">
    <GeoJSON data={gj} generateId>
      <FillLayer
        {...layerId("debug-demand-fill")}
        paint={{
          "fill-color": fillColor,
          "fill-opacity": hoverStateFilter(0.5, 0.1),
        }}
        manageHoverState
        bind:hovered
      />

      <LineLayer
        {...layerId("debug-demand-outline")}
        paint={{
          "line-width": 2,
          "line-color": "black",
        }}
      />
    </GeoJSON>
  </div>
</SplitComponent>
