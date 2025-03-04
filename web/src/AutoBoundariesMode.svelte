<script lang="ts">
  import type {
    DataDrivenPropertyValueSpecification,
    ExpressionSpecification,
  } from "maplibre-gl";
  import {
    FillLayer,
    GeoJSON,
    hoverStateFilter,
    LineLayer,
    type LayerClickInfo,
  } from "svelte-maplibre";
  import { downloadGeneratedFile } from "svelte-utils";
  import { makeRamp, Popup } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import { layerId, Link } from "./common";
  import {
    areaLimits,
    simdColorScale,
    simdLimits,
    stats19ColorScale,
    stats19Limits,
  } from "./common/colors";
  import { pickNeighbourhoodName } from "./common/pick_names";
  import { PrioritizationSelect, type Prioritization } from "./prioritization";
  import {
    autosave,
    backend,
    editPerimeterRoads,
    mode,
    projectName,
  } from "./stores";
  import type { GeneratedBoundaryFeature } from "./wasm";

  let generatedBoundaries = $backend!.generatedBoundaries();
  let minArea = 0;
  let removeNonRoad = true;
  let selectedPrioritization: Prioritization = "none";

  function clickedBoundary(e: CustomEvent<LayerClickInfo>) {
    // this isn't quite right - there's no .waypoint field. I was expect: `waypoints: []`
    let feature: GeneratedBoundaryFeature =
      generatedBoundaries.features[e.detail.features[0].id as number];
    console.log("clicked feature", feature);
    createNeighbourhood(feature);
  }

  function createNeighbourhood(selectedBoundary: GeneratedBoundaryFeature) {
    let name = pickNeighbourhoodName(
      $backend!,
      "What do you want to name the neighbourhood?",
      "",
    );
    if (!name) {
      return;
    }
    try {
      let feature = {
        type: "Feature" as const,
        // Omit waypoints and lazily fill them out.
        properties: {},
        // Trust generateId to make IDs in order
        geometry: selectedBoundary.geometry,
      };
      $backend!.setNeighbourhoodBoundary(name, feature);
      autosave();
      $backend!.setCurrentNeighbourhood(name, $editPerimeterRoads);
      $mode = {
        mode: "neighbourhood",
      };
    } catch (err) {
      console.log("error when setting auto-generated neighborhood", err);
      window.alert(
        "Sorry, an error occurred. You may need to refresh the page for the application to continue working.",
      );
    }
  }

  function download() {
    downloadGeneratedFile(
      "auto_boundaries.geojson",
      JSON.stringify(generatedBoundaries, null, "  "),
    );
  }

  function makeFilter(
    minArea: number,
    removeNonRoad: boolean,
  ): ExpressionSpecification {
    let x: ExpressionSpecification = [
      "all",
      [">=", ["get", "area_km2"], minArea],
    ];
    if (removeNonRoad) {
      x.push(["get", "touches_big_road"]);
    }
    return x;
  }

  function fillColor(
    selectedPrioritization: Prioritization,
  ): DataDrivenPropertyValueSpecification<string> {
    return {
      none: [
        "match",
        ["%", ["id"], 5],
        0,
        "blue",
        1,
        "yellow",
        2,
        "green",
        3,
        "purple",
        4,
        "orange",
        "black",
      ] as DataDrivenPropertyValueSpecification<string>,
      simd: makeRamp(["get", "simd"], simdLimits, simdColorScale),
      area: makeRamp(["get", "area_km2"], areaLimits, simdColorScale),
      stats19: makeRamp(
        ["/", ["get", "number_stats19_collisions"], ["get", "area_km2"]],
        stats19Limits,
        stats19ColorScale,
      ),
    }[selectedPrioritization];
  }

  function fillOpacity(
    selectedPrioritization: Prioritization,
  ): DataDrivenPropertyValueSpecification<number> {
    return {
      none: hoverStateFilter(0.3, 0.5),
      simd: hoverStateFilter(0.7, 0.9),
      area: hoverStateFilter(0.7, 0.9),
      stats19: hoverStateFilter(0.7, 0.9),
    }[selectedPrioritization];
  }
</script>

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <Link on:click={() => ($mode = { mode: "title", firstLoad: false })}>
            Choose project
          </Link>
        </li>
        <li>
          <Link on:click={() => ($mode = { mode: "pick-neighbourhood" })}>
            Pick neighbourhood
          </Link>
        </li>
        <li>Use an auto-generated boundary</li>
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <BackButton on:click={() => ($mode = { mode: "pick-neighbourhood" })} />

    <p>
      Click an area to use it as the boundary for your neighbourhood. These
      particular boundaries are suggested by finding roads, railways, and water
      that form severances.
    </p>

    {#if $projectName.startsWith("ltn_cnt/")}
      <h3>Prioritization</h3>
      <p>Compare metrics across candidate neighbourhoods.</p>
      <PrioritizationSelect bind:selectedPrioritization />
    {/if}

    {#if selectedPrioritization == "none"}
      <p>The colors are arbitrary, just to distinguish better.</p>
    {/if}

    <hr />

    <button class="secondary" on:click={download}>Export to GeoJSON</button>

    <label>
      Minimum area (km²)
      <input type="number" bind:value={minArea} min="0" max="1" step="0.01" />
    </label>

    <label>
      <input type="checkbox" bind:checked={removeNonRoad} />
      Remove areas not touching a big road
    </label>
  </div>

  <div slot="map">
    <GeoJSON data={generatedBoundaries} generateId>
      <FillLayer
        {...layerId("neighbourhood-boundaries", false)}
        filter={makeFilter(minArea, removeNonRoad)}
        paint={{
          "fill-color": fillColor(selectedPrioritization),
          "fill-opacity": fillOpacity(selectedPrioritization),
        }}
        manageHoverState
        hoverCursor="pointer"
        on:click={clickedBoundary}
      >
        <Popup openOn="hover" let:props>
          {#if selectedPrioritization == "none" || selectedPrioritization == "area"}
            <b>Area:</b>
            {props.area_km2.toFixed(1)} km²
          {:else if selectedPrioritization == "simd"}
            <b>SIMD:</b>
            Less deprived than {props.simd.toFixed(1)}% of data zones.
          {:else if selectedPrioritization == "stats19"}
            <b>
              Density of pedestrian and cyclist collisions (collisions per
              square kilometer):
            </b>
            {(props.number_stats19_collisions / props.area_km2).toFixed(1)}
          {/if}
        </Popup>
      </FillLayer>

      <LineLayer
        {...layerId("neighbourhood-boundaries-outline", false)}
        filter={makeFilter(minArea, removeNonRoad)}
        paint={{
          "line-color": "black",
          "line-width": 1,
        }}
      />
    </GeoJSON>
  </div>
</SplitComponent>
