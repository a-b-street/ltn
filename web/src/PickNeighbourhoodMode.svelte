<script lang="ts">
  import type { FeatureCollection } from "geojson";
  import { CirclePlus, FileDown, Pencil, Trash2 } from "lucide-svelte";
  import { type DataDrivenPropertyValueSpecification } from "maplibre-gl";
  import {
    FillLayer,
    GeoJSON,
    hoverStateFilter,
    JoinedData,
    LineLayer,
  } from "svelte-maplibre";
  import { downloadGeneratedFile, notNull } from "svelte-utils";
  import { Popup } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { downloadProject, HelpButton, layerId, Link, Style } from "./common";
  import { pickNeighbourhoodName } from "./common/pick_names";
  import { ModalFilterLayer } from "./layers";
  import {
    prioritizationFillColor,
    PrioritizationSelect,
    type Prioritization,
  } from "./prioritization";
  import {
    appFocus,
    backend,
    currentProjectID,
    devMode,
    mode,
    projectStorage,
    returnToChooseProject,
    saveCurrentProject,
  } from "./stores";
  import type { NeighbourhoodBoundaryFeature } from "./wasm";

  // Note we do this to trigger a refresh when loading stuff
  $: neighbourhoods = $backend!.getAllNeighbourhoods();
  $: edits = countEdits(neighbourhoods);

  let selectedPrioritization: Prioritization = "none";
  let hoveredNeighbourhoodFromList: string | null = null;
  let hoveredMapFeature: NeighbourhoodBoundaryFeature | null = null;

  function pickNeighbourhood(name: string) {
    $backend!.setCurrentNeighbourhood(name);
    $mode = { mode: "neighbourhood" };
  }

  function deleteNeighbourhood(name: string) {
    if (
      window.confirm(
        `Really delete neighbourhood ${name}? You can't undo this.`,
      )
    ) {
      $backend!.deleteNeighbourhoodBoundary(name);
      saveCurrentProject();
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
      saveCurrentProject();
      neighbourhoods = $backend!.getAllNeighbourhoods();
    }
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
    let highlightedColor = Style.mapFeature.hover.backgroundColor;
    let color = prioritizationFillColor(
      { none: highlightedColor },
      selectedPrioritization,
    );
    return [
      "case",
      ["==", ["feature-state", "highlight"], "yes"],
      highlightedColor,
      // @ts-expect-error MapLibre expression types are weird, but this really does work
      color,
    ];
  }

  function fillOpacity(
    selectedPrioritization: Prioritization,
  ): DataDrivenPropertyValueSpecification<number> {
    let highlightedOpacity = 0.7;
    let styles: Record<string, DataDrivenPropertyValueSpecification<number>> = {
      none: [
        "case",
        ["==", ["feature-state", "highlight"], "yes"],
        highlightedOpacity,
        hoverStateFilter(0.3, highlightedOpacity),
      ],
      area: hoverStateFilter(0.7, 0.9),
      density: hoverStateFilter(0.7, 0.9),
      simd: hoverStateFilter(0.7, 0.9),
      stats19: hoverStateFilter(0.7, 0.9),
      pois: hoverStateFilter(0.7, 0.9),
      car_ownership: hoverStateFilter(0.7, 0.9),
    };
    return styles[selectedPrioritization];
  }
</script>

<SplitComponent>
  <div slot="top" style="display: flex; justify-content: space-between;">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <Link on:click={returnToChooseProject}>Choose project</Link>
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
        {#if $devMode}
          <li>
            <Link on:click={() => ($mode = { mode: "debug-intersections" })}>
              Debug intersections
            </Link>
          </li>
        {/if}
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <div
      style="display: flex; justify-content: space-between; align-items: center; gap: 16px;"
    >
      <h1>{$projectStorage.projectName(notNull($currentProjectID))}</h1>
      <button
        class="outline icon-btn"
        style="margin-right: 8px;"
        title="Download project as GeoJSON"
        on:click={() => downloadProject(notNull($currentProjectID))}
      >
        <div
          style="display: flex; align-items: center; gap: 8px; color: black;"
        >
          <FileDown />
          <!-- 
            The text feels a little crowded aginst the right edge. 
            Currently this is the only place we use an icon+text button like this.
            But if we do more, we might want to pattern something out.
          -->
          <span style="margin-right: 2px;">Export</span>
        </div>
      </button>
    </div>
    <h2>Neighbourhoods</h2>
    <ul class="navigable-list">
      {#each neighbourhoods.features as { properties: { name } }}
        <li
          on:mouseenter={() => (hoveredNeighbourhoodFromList = name)}
          on:mouseleave={() => (hoveredNeighbourhoodFromList = null)}
          class="actionable-cell"
          class:highlighted={hoveredMapFeature?.properties.name == name ||
            hoveredNeighbourhoodFromList == name}
        >
          <h3><Link on:click={() => pickNeighbourhood(name)}>{name}</Link></h3>
          <span class="actions">
            <button
              class="outline icon-btn"
              title="Rename neighbourhood"
              on:click={() => renameNeighbourhood(name)}
            >
              <Pencil color="black" />
            </button>
            <button
              class="icon-btn destructive"
              title="Delete neighbourhood"
              on:click={() => deleteNeighbourhood(name)}
            >
              <Trash2 color="white" />
            </button>
          </span>
        </li>
      {/each}
      <li>
        <Link on:click={() => ($mode = { mode: "add-neighbourhood" })}>
          <CirclePlus />
          Add a new neighbourhood
        </Link>
      </li>
    </ul>

    {#if $appFocus == "cnt"}
      <h3>Prioritization</h3>
      <p>Compare metrics across your neighbourhoods.</p>
      <PrioritizationSelect bind:selectedPrioritization />
      <hr />
    {/if}

    <p>
      {edits.modalFilters} new modal filter(s) added
    </p>
    <p>
      {edits.deletedModalFilters}
      existing modal filter(s) removed
    </p>
    <p>{edits.travelFlows} road segment direction(s) changed</p>

    {#if $devMode}
      <button class="secondary" on:click={debugRouteSnapper}>
        Debug route-snapper
      </button>
    {/if}
  </div>

  <div slot="map">
    <GeoJSON data={neighbourhoods} promoteId="name">
      <JoinedData
        data={hoveredNeighbourhoodFromList
          ? [{ name: hoveredNeighbourhoodFromList, highlight: "yes" }]
          : []}
        idCol="name"
      />

      <LineLayer
        filter={["==", ["get", "kind"], "boundary"]}
        {...layerId("neighbourhood-boundaries-selected-outline", false)}
        manageHoverState={false}
        paint={{
          "line-color": "black",
          "line-width": 4,
          "line-dasharray": [2, 2],
        }}
      />
      <LineLayer
        filter={["==", ["get", "kind"], "boundary"]}
        {...layerId("neighbourhood-boundaries-selected-outline-base", false)}
        manageHoverState={false}
        paint={{
          "line-color": "white",
          "line-width": 8,
          "line-opacity": 0.7,
        }}
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

          {#if selectedPrioritization == "density"}
            <b>Population density:</b>
            {Math.round(props.population / props.area_km2).toLocaleString()} people
            / km²
          {:else if selectedPrioritization == "stats19"}
            <b>Pedestrian and cyclist collisions:</b>
            {(props.number_stats19_collisions / props.area_km2).toFixed(1)} / km²
          {:else if selectedPrioritization == "pois"}
            <b>Points of interest:</b>
            {(props.number_pois / props.area_km2).toFixed(1)} / km²
          {/if}
        </Popup>
      </FillLayer>
    </GeoJSON>

    <ModalFilterLayer interactive={false} />
  </div>
</SplitComponent>

<style>
  li.highlighted {
    background-color: rgba(72, 96, 202, 0.15);
  }
</style>
