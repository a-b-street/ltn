<script lang="ts">
  import type { FeatureCollection } from "geojson";
  import { CirclePlus, Pencil, Trash2 } from "lucide-svelte";
  import type { DataDrivenPropertyValueSpecification } from "maplibre-gl";
  import {
    FillLayer,
    GeoJSON,
    hoverStateFilter,
    JoinedData,
  } from "svelte-maplibre";
  import { downloadGeneratedFile, notNull } from "svelte-utils";
  import { makeRamp, Popup } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { HelpButton, layerId, Link } from "./common";
  import {
    areaLimits,
    simdColorScale,
    simdLimits,
    stats19ColorScale,
    stats19Limits,
  } from "./common/colors";
  import { pickNeighbourhoodName } from "./common/pick_names";
  import { ModalFilterLayer } from "./layers";
  import { PrioritizationSelect, type Prioritization } from "./prioritization";
  import {
    autosave,
    backend,
    editPerimeterRoads,
    mode,
    projectName,
  } from "./stores";
  import type { NeighbourhoodBoundaryFeature } from "./wasm";

  // Note we do this to trigger a refresh when loading stuff
  $: neighbourhoods = $backend!.getAllNeighbourhoods();
  $: edits = countEdits(neighbourhoods);

  let selectedPrioritization: Prioritization = "none";
  let hoveredNeighbourhoodFromList: string | null = null;
  let hoveredMapFeature: NeighbourhoodBoundaryFeature | null = null;

  function pickNeighbourhood(name: string) {
    $backend!.setCurrentNeighbourhood(name, $editPerimeterRoads);
    $mode = { mode: "neighbourhood" };
  }

  function deleteNeighbourhood(name: string) {
    if (
      window.confirm(
        `Really delete neighbourhood ${name}? You can't undo this.`,
      )
    ) {
      $backend!.deleteNeighbourhoodBoundary(name);
      autosave();
      // TODO Improve perf, don't call this twice
      neighbourhoods = $backend!.getAllNeighbourhoods();
    }
  }

  function renameNeighbourhood(name: string) {
    let newName = pickNeighbourhoodName(
      $backend!,
      `Rename neighbourhood ${name} to what?`,
      name,
    );
    if (newName) {
      $backend!.renameNeighbourhoodBoundary(name, newName);
      autosave();
      neighbourhoods = $backend!.getAllNeighbourhoods();
    }
  }

  function newBoundary() {
    let name = pickNeighbourhoodName(
      $backend!,
      "What do you want to name the neighbourhood?",
      "",
    );
    if (name) {
      $mode = { mode: "set-boundary", name, existing: null };
    }
  }

  function exportGJ() {
    downloadGeneratedFile(
      $projectName + ".geojson",
      JSON.stringify($backend!.toSavefile()),
    );
  }

  function debugRouteSnapper() {
    downloadGeneratedFile(
      "debug_route_snapper.geojson",
      $backend!.toRouteSnapperGj(),
    );
  }

  function countEdits(gj: FeatureCollection): {
    modalFilters: number;
    deletedModalFilters: number;
    travelFlows: number;
  } {
    let modalFilters = 0;
    let deletedModalFilters = 0;
    let travelFlows = 0;
    for (let f of gj.features) {
      if (f.properties!.kind == "modal_filter") {
        modalFilters++;
      } else if (f.properties!.kind == "deleted_existing_modal_filter") {
        deletedModalFilters++;
      } else if (f.properties!.kind == "travel_flow") {
        travelFlows++;
      }
    }
    return { modalFilters, deletedModalFilters, travelFlows };
  }

  function fillColor(
    selectedPrioritization: Prioritization,
  ): DataDrivenPropertyValueSpecification<string> {
    let color = {
      none: "black",
      simd: makeRamp(["get", "simd"], simdLimits, simdColorScale),
      area: makeRamp(["get", "area_km2"], areaLimits, simdColorScale),
      stats19: makeRamp(
        ["/", ["get", "number_stats19_collisions"], ["get", "area_km2"]],
        stats19Limits,
        stats19ColorScale,
      ),
    }[selectedPrioritization];
    return [
      "case",
      ["==", ["feature-state", "highlight"], "yes"],
      "yellow",
      // @ts-expect-error MapLibre expression types are weird, but this really does work
      color,
    ];
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
  <div slot="top" style="display: flex; justify-content: space-between;">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <Link on:click={() => ($mode = { mode: "title", firstLoad: false })}>
            Choose project
          </Link>
        </li>
        <li>
          Pick neighbourhood
          <HelpButton>
            <p>
              Inside the neighbourhood you define, the goal is to eliminate (or
              deliberately permit) through-traffic. An appropriate neighbourhood
              boundary depends on many factors. The simplest approach is to find
              the area bounded on all sides by "main" roads, which are designed
              for higher traffic volumes. There are many other considerations,
              though -- maybe severances like rivers or rail should be part of a
              boundary. Bridges and tunnels near a boundary may be confusing as
              well. And note that your boundary may not match the conventional
              definition of "neighbourhood."
            </p>
          </HelpButton>
        </li>
      </ul>
    </nav>
    <nav>
      <ul>
        <li>
          <Link
            on:click={() =>
              ($mode = { mode: "route", prevMode: "pick-neighbourhood" })}
          >
            Route
          </Link>
        </li>
        <li>
          <Link on:click={() => ($mode = { mode: "predict-impact" })}>
            Predict impact
          </Link>
        </li>
        <li>
          <Link on:click={() => ($mode = { mode: "debug-intersections" })}>
            Debug intersections
          </Link>
        </li>
        <li>
          <Link on:click={() => ($mode = { mode: "debug-demand" })}>
            Debug demand
          </Link>
        </li>
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <h3 style="padding: 4px 8px; margin-bottom: 0">Neighbourhoods</h3>
    <ul class="neighbourhood-list">
      {#each neighbourhoods.features as { properties: { name, area_km2, simd } }}
        <li
          on:mouseenter={() => (hoveredNeighbourhoodFromList = name)}
          on:mouseleave={() => (hoveredNeighbourhoodFromList = null)}
          class:highlighted={hoveredMapFeature?.properties.name == name}
        >
          <span style="display: flex; justify-content: space-between;">
            <span class="neighbourhood-name">
              <Link on:click={() => pickNeighbourhood(name)}>
                {name}
              </Link>
            </span>
            <span style="display: flex; gap: 16px;">
              <button
                class="outline icon-btn"
                aria-label="Rename neighbourhood"
                on:click={() => renameNeighbourhood(name)}
              >
                <Pencil color="black" />
              </button>
              <button
                class="icon-btn destructive"
                aria-label="Delete neighbourhood"
                on:click={() => deleteNeighbourhood(name)}
              >
                <Trash2 color="white" />
              </button>
            </span>
          </span>
          {#if selectedPrioritization == "area"}
            <span>
              <b>Area:</b>
              {area_km2.toFixed(1)} km²
            </span>
          {:else if selectedPrioritization == "simd"}
            <span>
              <b>SIMD (percentile):</b>
              {simd.toFixed(1)}
            </span>
          {/if}
        </li>
      {/each}
      <li>
        Add a new neighbourhood:
        <ul style="margin-bottom: 0; padding: 0;">
          <!-- pico override -->
          <li style="list-style: none;">
            <Link on:click={newBoundary}>
              <Pencil />
              Draw a new boundary
            </Link>
          </li>
          <li style="list-style: none;">
            <Link on:click={() => ($mode = { mode: "auto-boundaries" })}>
              <CirclePlus />
              Use a generated boundary
            </Link>
          </li>
        </ul>
      </li>
    </ul>

    {#if $projectName.startsWith("ltn_cnt/")}
      <h3>Prioritization</h3>
      <p>Compare metrics across your neighbourhoods.</p>
      <PrioritizationSelect bind:selectedPrioritization />
      <hr />
    {/if}

    <p>Current project: {$projectName}</p>

    <p>
      {edits.modalFilters} new modal filter(s) added
    </p>
    <p>
      {edits.deletedModalFilters}
      existing modal filter(s) removed
    </p>
    <p>{edits.travelFlows} road segment direction(s) changed</p>

    <button on:click={exportGJ}>Export project to GeoJSON</button>
    <button class="secondary" on:click={debugRouteSnapper}>
      Debug route-snapper
    </button>
  </div>

  <div slot="map">
    <GeoJSON data={neighbourhoods} promoteId="name">
      <JoinedData
        data={hoveredNeighbourhoodFromList
          ? [{ name: hoveredNeighbourhoodFromList, highlight: "yes" }]
          : []}
        idCol="name"
      />

      <FillLayer
        {...layerId("neighbourhood-boundaries", false)}
        filter={["==", ["get", "kind"], "boundary"]}
        paint={{
          "fill-color": fillColor(selectedPrioritization),
          "fill-opacity": fillOpacity(selectedPrioritization),
        }}
        manageHoverState
        bind:hovered={hoveredMapFeature}
        hoverCursor="pointer"
        on:click={(e) =>
          pickNeighbourhood(notNull(e.detail.features[0].properties).name)}
      >
        <Popup openOn="hover" let:props>
          <h2>{props.name}</h2>

          {#if selectedPrioritization == "none" || selectedPrioritization == "area"}
            <b>Area:</b>
            {props.area_km2.toFixed(1)} km²
          {:else if selectedPrioritization == "simd"}
            <b>Fake SIMD:</b>
            {props.simd.toFixed(1)}
          {:else if selectedPrioritization == "stats19"}
            <b>
              Density of pedestrian and cyclist collisions (collisions per
              square kilometer):
            </b>
            {(props.number_stats19_collisions / props.area_km2).toFixed(1)}
          {/if}
        </Popup>
      </FillLayer>
    </GeoJSON>

    <ModalFilterLayer />
  </div>
</SplitComponent>

<style>
  ul.neighbourhood-list {
    padding: 0px;
  }

  ul.neighbourhood-list > li {
    list-style: none;
    margin: 0;
    padding: 16px 8px;

    display: flex;
    flex-direction: column;
    align-items: leading;
    width: 100%;
    border-bottom: solid #ddd 1px;
  }

  ul.neighbourhood-list > li.highlighted {
    background-color: #f0fcaa;
  }

  ul.neighbourhood-list .neighbourhood-name {
    font-weight: bold;
    font-size: 1.2em;
  }
</style>
