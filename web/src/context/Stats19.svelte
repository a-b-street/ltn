<script lang="ts">
  import type { ExpressionSpecification } from "maplibre-gl";
  import {
    CircleLayer,
    VectorTileSource,
    type LayerClickInfo,
  } from "svelte-maplibre";
  import { QualitativeLegend } from "svelte-utils";
  import { makeRamp, Popup } from "svelte-utils/map";
  import { layerId } from "../common";
  import ContextLayerButton from "../common/ContextLayerButton.svelte";
  import { assetUrl } from "../stores";

  let show = false;
  let state = {
    pedestrians: true,
    cyclists: true,
    minYear: 2017,
    maxYear: 2023,
  };

  function makeFilter(_: any): ExpressionSpecification {
    let includeTypes: ExpressionSpecification = ["any"];
    if (state.pedestrians) {
      includeTypes.push(["get", "pedestrian"]);
    }
    if (state.cyclists) {
      includeTypes.push(["get", "cyclist"]);
    }

    return [
      "all",
      [">=", ["get", "year"], state.minYear],
      ["<=", ["get", "year"], state.maxYear],
      includeTypes,
    ];
  }

  function casualtyTypes(props: { [name: string]: any }): string {
    let list = [];
    if (props.pedestrian) {
      list.push("pedestrian");
    }
    if (props.cyclist) {
      list.push("cyclist");
    }
    return list.join(", ");
  }

  function onClick(e: CustomEvent<LayerClickInfo>) {
    window.open(
      `https://www.cyclestreets.net/collisions/reports/${
        e.detail.features[0].properties!.accident_index
      }`,
      "_blank",
    );
  }

  // From
  // https://data.dft.gov.uk/road-accidents-safety-data/dft-road-casualty-statistics-road-safety-open-dataset-data-guide-2023.xlsx.
  // The "not a pedestrian", "data missing", and "unknown" cases are filtered out
  // upstream.
  let pedestrianLocation: { [name: number]: string } = {
    1: "Crossing on pedestrian crossing facility",
    2: "Crossing in zig-zag approach lines",
    3: "Crossing in zig-zag exit lines",
    4: "Crossing elsewhere within 50m. of pedestrian crossing",
    5: "In carriageway, crossing elsewhere",
    6: "On footway or verge",
    7: "On refuge, central island or central reservation",
    8: "In centre of carriageway - not on refuge, island or central reservation",
    9: "In carriageway, not crossing",
  };
  let pedestrianMovement: { [name: number]: string } = {
    1: "Crossing from driver's nearside",
    2: "Crossing from nearside - masked by parked or stationary vehicle",
    3: "Crossing from driver's offside",
    4: "Crossing from offside - masked by parked or stationary vehicle",
    5: "In carriageway, stationary - not crossing (standing or playing)",
    6: "In carriageway, stationary - not crossing (standing or playing - masked by parked or stationary vehicle)",
    7: "Walking along in carriageway, facing traffic",
    8: "Walking along in carriageway, back to traffic",
  };
  let severity: { [name: number]: string } = {
    1: "Fatal",
    2: "Serious",
    3: "Slight",
  };

  let fatalColor = "#080C54";
  let seriousColor = "#1F9EB7";
  let slightColor = "#CDE594";
  let severityLegend = {
    Fatal: fatalColor,
    Serious: seriousColor,
    Slight: slightColor,
  };
</script>

<ContextLayerButton bind:show label="Collisions">
  <div slot="help">
    <p>
      This layer shows collisions recorded in the <a
        href="https://www.data.gov.uk/dataset/cb7ae6f0-4be6-4935-9277-47e5ce24a11f/road-safety-data"
        target="_blank"
      >
        DfT stats19
      </a>
      dataset, as of 30 September 2024. Please note these limitations:
    </p>
    <ul>
      <li>Only collisions between 2017 and 2023 are included</li>
      <li>
        This tool is intended to be used when zoomed into the map, while
        inspecting a scheme or development area. Not all points are shown when
        zoomed out and showing large areas. Do not use this to look for trends
        across a city or region scale.
      </li>
      <li>
        Approximately 150 collisions from the source data aren't included, due
        to problems with the recorded location
      </li>
      <li>The "pedestrians" category also include mobility scooters</li>
      <li>
        All limitations <a
          href="https://www.gov.uk/guidance/road-accident-and-safety-statistics-guidance"
          target="_blank"
        >
          documented by DfT
        </a>
        also apply. Not all collisions or near misses are reported. There's nuance
        with the severity categories.
      </li>
    </ul>
    <p>
      You can click a point to open the full report, thanks to CycleStreets.
    </p>

    <p>
      License: <a
        href="http://www.nationalarchives.gov.uk/doc/open-government-licence/version/3/"
        target="_blank"
      >
        Open Government License
      </a>
      . Contains OS data &copy; Crown copyright and database right 2025.
    </p>
  </div>
  <div slot="legend">
    <fieldset style="display: flex; gap: 3em;">
      <label>
        <input type="checkbox" bind:checked={state.pedestrians} />
        Pedestrians
      </label>
      <label>
        <input type="checkbox" bind:checked={state.cyclists} />
        Cyclists
      </label>
    </fieldset>
    <fieldset class="year-filter">
      <label>
        From
        <input type="number" min={2017} max={2023} bind:value={state.minYear} />
      </label>
      <label>
        To
        <input type="number" min={2017} max={2023} bind:value={state.maxYear} />
      </label>
    </fieldset>
    <QualitativeLegend colors={severityLegend} horiz />
  </div>
</ContextLayerButton>

<VectorTileSource url={`pmtiles://${assetUrl("cnt_layers/stats19.pmtiles")}`}>
  <CircleLayer
    {...layerId("context-stats19")}
    sourceLayer="stats19"
    paint={{
      "circle-color": makeRamp(
        ["get", "severity"],
        [1, 2, 3],
        [fatalColor, seriousColor, slightColor],
      ),
      "circle-opacity": 0.9,
      "circle-radius": [
        "interpolate",
        ["linear"],
        ["zoom"],
        1,
        2,
        8,
        3,
        13,
        15,
      ],
      "circle-stroke-color": "black",
      "circle-stroke-width": 0.1,
    }}
    filter={makeFilter(state)}
    layout={{
      visibility: show ? "visible" : "none",
    }}
    hoverCursor="pointer"
    on:click={onClick}
  >
    <Popup let:props>
      <p>
        Year: <b>{props.year}</b>
      </p>
      <p>
        Severity: <b>{severity[props.severity]}</b>
      </p>
      <p>
        Casualties: <b>{casualtyTypes(props)}</b>
      </p>
      {#if props.pedestrian_location}
        <p>
          Pedestrian location: <b>
            {pedestrianLocation[props.pedestrian_location]}
          </b>
        </p>
      {/if}
      {#if props.pedestrian_movement}
        <p>
          Pedestrian movement: <b>
            {pedestrianMovement[props.pedestrian_movement]}
          </b>
        </p>
      {/if}
      <p>Click to open full report in CycleStreets</p>
    </Popup>
  </CircleLayer>
</VectorTileSource>

<style>
  .year-filter {
    display: flex;
    gap: 3em;
  }
  input,
  fieldset {
    margin: 0;
  }
  input {
    margin: 8px 0;
  }

  .year-filter input {
    height: 36px;
  }
</style>
