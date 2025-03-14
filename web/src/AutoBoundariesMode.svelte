<script lang="ts">
  import { CircleX } from "lucide-svelte";
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
  import { downloadGeneratedFile, notNull } from "svelte-utils";
  import { Popup } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import { layerId, Link, prettyPrintPercent } from "./common";
  import { pickNeighbourhoodName } from "./common/pick_names";
  import {
    prioritizationFillColor,
    PrioritizationSelect,
    type Prioritization,
  } from "./prioritization";
  import {
    appFocus,
    autosave,
    backend,
    mode,
    returnToChooseProject,
  } from "./stores";
  import type { GeneratedBoundaryFeature } from "./wasm";

  let generatedBoundaries = $backend!.generatedBoundaries();
  let minArea = 0;
  let removeNonRoad = true;
  let selectedPrioritization: Prioritization = "none";
  let selectedBoundary: GeneratedBoundaryFeature | null = null;
  let selectedBoundaries: Map<number, GeneratedBoundaryFeature> = new Map();

  function clickedBoundary(e: CustomEvent<LayerClickInfo>) {
    // Trust generateId to make IDs in order
    let featureId = e.detail.features[0].id as number;
    let feature: GeneratedBoundaryFeature =
      generatedBoundaries.features[featureId];

    let newBoundaries = new Map(selectedBoundaries);

    // toggle
    if (newBoundaries.has(featureId)) {
      newBoundaries.delete(featureId);
    } else {
      newBoundaries.set(featureId, feature);
    }

    if (newBoundaries.size === 0) {
      selectedBoundary = null;
      selectedBoundaries = newBoundaries;
    } else if (newBoundaries.size === 1) {
      selectedBoundary = newBoundaries.values().next().value!;
      selectedBoundaries = newBoundaries;
    } else {
      // Aggregate the selected boundaries
      let featureCollection = {
        type: "FeatureCollection" as const,
        features: Array.from(newBoundaries.values()),
      };
      try {
        selectedBoundary = $backend!.generateMergedBoundary(featureCollection);
        selectedBoundaries = newBoundaries;
      } catch (error) {
        console.log(`error merging boundaries: ${error}`);
        window.alert(
          `Sorry, the boundaries you selected cannot be merged.\nError: ${error}`,
        );
      }
    }
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
        geometry: selectedBoundary.geometry,
      };
      $backend!.setNeighbourhoodBoundary(name, feature);
      autosave();
      $backend!.setCurrentNeighbourhood(name);
      $mode = {
        mode: "neighbourhood",
      };
    } catch (err) {
      window.alert(`Sorry, this boundary is invalid: ${err}`);
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
    let noneColor = [
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
    ] as DataDrivenPropertyValueSpecification<string>;

    return prioritizationFillColor({ none: noneColor }, selectedPrioritization);
  }

  function fillOpacity(
    selectedPrioritization: Prioritization,
  ): DataDrivenPropertyValueSpecification<number> {
    return {
      none: hoverStateFilter(0.3, 0.5),
      area: hoverStateFilter(0.7, 0.9),
      car_ownership: hoverStateFilter(0.7, 0.9),
      density: hoverStateFilter(0.7, 0.9),
      pois: hoverStateFilter(0.7, 0.9),
      simd: hoverStateFilter(0.7, 0.9),
      stats19: hoverStateFilter(0.7, 0.9),
    }[selectedPrioritization];
  }
</script>

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <Link on:click={returnToChooseProject}>Choose project</Link>
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
    {#if $appFocus == "cnt"}
      <h3>Prioritization</h3>
      <p>Compare metrics across candidate neighbourhoods.</p>
      <PrioritizationSelect bind:selectedPrioritization />
    {/if}

    {#if selectedPrioritization == "none"}
      <p>The colors are arbitrary, just to distinguish better.</p>
    {/if}

    <hr />

    <div>
      <h3>1. Choose area</h3>
      <div style="border: dashed black 2px; padding: 8px">
        {#if selectedBoundary}
          <div style="display: flex; justify-content: space-between;">
            <b>Your neighbourhood overall</b>
            <button
              class="icon-btn destructive"
              aria-label="Clear selection"
              on:click={() => {
                selectedBoundary = null;
                selectedBoundaries.clear();
                selectedBoundaries = selectedBoundaries;
              }}
            >
              <CircleX />
            </button>
          </div>

          <table>
            <tr>
              <th>Area</th>
              <td>
                {selectedBoundary.properties.area_km2.toFixed(1)} km²
              </td>
            </tr>
            {#if $appFocus == "cnt"}
              <tr>
                <th>Population density</th>
                <td>
                  {Math.round(
                    selectedBoundary.properties.population /
                      selectedBoundary.properties.area_km2,
                  ).toLocaleString()} people / km²
                </td>
              </tr>
              <tr>
                <th>SIMD</th>
                <td>{selectedBoundary.properties.simd.toFixed(1)}%</td>
              </tr>
              <tr>
                <th>Collision density</th>
                <td>
                  {(
                    selectedBoundary.properties.number_stats19_collisions /
                    selectedBoundary.properties.area_km2
                  ).toFixed(1)} / km²
                </td>
              </tr>
              <tr>
                <th>Car ownership</th>
                <td>
                  {prettyPrintPercent(
                    selectedBoundary.properties.households_with_cars_or_vans,
                    selectedBoundary.properties.total_households,
                  )} of households
                </td>
              </tr>
              <tr>
                <th>POI density</th>
                <td>
                  {(
                    selectedBoundary.properties.number_pois /
                    selectedBoundary.properties.area_km2
                  ).toFixed(1)} / km²
                </td>
              </tr>
            {/if}
          </table>
        {:else}
          <p>Choose an area to use as the boundary for your neighbourhood.</p>
          <p>
            These particular boundaries are suggested by finding roads,
            railways, and water that form severances.
          </p>
        {/if}
        {#if selectedBoundary}
          <h3>2. Add to area</h3>
          <p>Choose any adjacent areas you'd like to add to your boundary.</p>

          <h3>3. Finished?</h3>
          <p>When you're done, click "Confirm".</p>
          <button
            on:click={() => createNeighbourhood(notNull(selectedBoundary))}
          >
            Confirm Boundary
          </button>
        {/if}
      </div>
    </div>

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
    {#if selectedBoundary}
      <GeoJSON
        data={{
          type: "FeatureCollection",
          features: [selectedBoundary],
        }}
      >
        <LineLayer
          {...layerId("neighbourhood-boundaries-selected-outline", false)}
          paint={{
            "line-color": "black",
            "line-width": 4,
            "line-dasharray": [2, 2],
          }}
        />
        <LineLayer
          {...layerId("neighbourhood-boundaries-selected-outline-base", false)}
          paint={{
            "line-color": "white",
            "line-width": 8,
            "line-opacity": 0.7,
          }}
        />
      </GeoJSON>
    {/if}
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
        {#if selectedPrioritization != "simd"}
          <Popup openOn="hover" let:props>
            {#if selectedPrioritization == "none"}
              <b>Area:</b>
              {props.area_km2.toFixed(1)} km²
            {:else if selectedPrioritization == "car_ownership"}
              <b>Car or van ownership</b>
              <br />
              {prettyPrintPercent(
                props.households_with_cars_or_vans,
                props.total_households,
              )} of approximately {props.total_households.toLocaleString()}
              households have at least one car or van.
            {:else if selectedPrioritization == "density"}
              <b>Population density</b>
              <br />
              {Math.round(props.population / props.area_km2).toLocaleString()} people
              / km²
            {:else if selectedPrioritization == "stats19"}
              <b>Pedestrian and cyclist collisions</b>
              <br />
              {(props.number_stats19_collisions / props.area_km2).toFixed(1)} / km²
            {:else if selectedPrioritization == "pois"}
              <b>Points of interest</b>
              <br />
              {(props.number_pois / props.area_km2).toFixed(1)} / km²
            {/if}
          </Popup>
        {/if}
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
