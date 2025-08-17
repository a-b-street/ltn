<script lang="ts">
  import type { FeatureCollection, Polygon } from "geojson";
  import { CirclePlus, Pencil, Trash2 } from "lucide-svelte";
  import type {
    DataDrivenPropertyValueSpecification,
    MapGeoJSONFeature,
  } from "maplibre-gl";
  import {
    FillLayer,
    GeoJSON,
    hoverStateFilter,
    JoinedData,
    LineLayer,
    Popup,
  } from "svelte-maplibre";
  import { emptyGeojson } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { layerId, Link, ModeLink, pageTitle, Style } from "../common";
  import { pickNeighbourhoodName } from "../common/pick_names";
  import { ModalFilterLayer } from "../layers";
  import {
    prioritizationFillColor,
    PrioritizationSelect,
    type Prioritization,
  } from "../prioritization";
  import {
    appFocus,
    backend,
    currentNeighbourhoodName,
    currentProjectID,
    devMode,
    metricBuckets,
    mode,
    mutationCounter,
    saveCurrentProject,
  } from "../stores";
  import type { NeighbourhoodBoundaryFeature } from "../wasm";
  import ManageProject from "./ManageProject.svelte";

  // Note we do this to trigger a refresh when loading stuff
  $: gj = $mutationCounter > 0 ? $backend!.toSavefile() : emptyGeojson();
  $: neighbourhoods = {
    type: "FeatureCollection" as const,
    features: gj.features.filter((f) => f.properties!.kind == "boundary"),
  } as FeatureCollection<Polygon, { name: string }>;

  // If a user loads an empty project or deletes all neighbourhoods, don't show
  // them an empty pick screen
  $: if (neighbourhoods.features.length == 0) {
    $mode = { mode: "add-neighbourhood" };
  }

  let selectedPrioritization: Prioritization =
    $appFocus == "cnt" ? "combined" : "none";
  let hoveredNeighbourhoodFromList: string | null = null;
  let hoveredMapFeature:
    | (NeighbourhoodBoundaryFeature & MapGeoJSONFeature)
    | undefined = undefined;
  $currentNeighbourhoodName = undefined;

  function pickNeighbourhood(name: string) {
    $backend!.setCurrentNeighbourhood(name);
    $currentNeighbourhoodName = name;
    $mode = { mode: "neighbourhood" };
  }

  function deleteNeighbourhood(name: string) {
    if (
      window.confirm(
        `Really delete neighbourhood ${name}? You can't undo this.`,
      )
    ) {
      $backend!.deleteNeighbourhoodBoundary(name);
      console.assert(!currentNeighbourhoodName);
      saveCurrentProject();
      $mutationCounter++;
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
      $currentNeighbourhoodName = newName;
      saveCurrentProject();
      $mutationCounter++;
    }
  }

  function fillColor(
    selectedPrioritization: Prioritization,
  ): DataDrivenPropertyValueSpecification<string> {
    let highlightedColor = Style.mapFeature.hover.backgroundColor;
    let color = prioritizationFillColor(
      { none: highlightedColor },
      selectedPrioritization,
      $metricBuckets,
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
    let styles: Record<
      Prioritization | "area",
      DataDrivenPropertyValueSpecification<number>
    > = {
      none: [
        "case",
        ["==", ["feature-state", "highlight"], "yes"],
        highlightedOpacity,
        hoverStateFilter(0.3, highlightedOpacity),
      ],
      area: hoverStateFilter(0.7, 0.9),
      population_density: hoverStateFilter(0.7, 0.9),
      simd: hoverStateFilter(0.7, 0.9),
      stats19: hoverStateFilter(0.7, 0.9),
      pois: hoverStateFilter(0.7, 0.9),
      car_ownership: hoverStateFilter(0.7, 0.9),
      combined: hoverStateFilter(0.7, 0.9),
    };
    return styles[selectedPrioritization];
  }
</script>

<SplitComponent>
  {#snippet top()}
    <div style="display: flex; justify-content: space-between;">
      <nav aria-label="breadcrumb">
        <ul>
          <li>
            <ModeLink mode={{ mode: "title" }} />
          </li>
          <li>
            {#key $currentProjectID}
              {pageTitle($mode.mode)}
            {/key}
          </li>
        </ul>
      </nav>
      <nav>
        <ul>
          <li>
            <ModeLink
              mode={{ mode: "route", prevMode: "pick-neighbourhood" }}
            />
          </li>
          <li>
            <ModeLink
              mode={{ mode: "predict-impact", prevMode: "pick-neighbourhood" }}
            />
          </li>
          {#if $devMode}
            <li>
              <ModeLink mode={{ mode: "debug-intersections" }} />
            </li>
          {/if}
        </ul>
      </nav>
    </div>
  {/snippet}

  {#snippet left()}
    <h2>Neighbourhoods</h2>
    <ul class="navigable-list">
      {#each neighbourhoods.features as { properties: { name } }}
        <li
          onmouseenter={() => (hoveredNeighbourhoodFromList = name)}
          onmouseleave={() => (hoveredNeighbourhoodFromList = null)}
          class="actionable-cell"
          class:highlighted={hoveredMapFeature?.properties.name == name ||
            hoveredNeighbourhoodFromList == name}
        >
          <h3><Link on:click={() => pickNeighbourhood(name)}>{name}</Link></h3>
          <span class="actions">
            <button
              class="outline icon-btn"
              title="Rename neighbourhood"
              onclick={() => renameNeighbourhood(name)}
            >
              <Pencil color="black" />
            </button>
            <button
              class="icon-btn destructive"
              title="Delete neighbourhood"
              onclick={() => deleteNeighbourhood(name)}
            >
              <Trash2 color="white" />
            </button>
          </span>
        </li>
      {/each}
      <li>
        <ModeLink mode={{ mode: "add-neighbourhood" }}>
          <CirclePlus />
          Add a new neighbourhood
        </ModeLink>
      </li>
    </ul>

    {#if $appFocus == "cnt"}
      <h3>Prioritisation</h3>
      <p>
        Compare the prioritisation or individual metrics across your
        neighbourhoods.
      </p>
      <PrioritizationSelect bind:selectedPrioritization />
      <hr />
    {/if}

    <ManageProject projectGj={gj} />
  {/snippet}

  {#snippet main()}
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
        onclick={(e) => pickNeighbourhood(e.features[0].properties!.name)}
      >
        <Popup openOn="hover">
          {#snippet children({ data })}
            {@const props = data!.properties!}
            <h2>{props.name}</h2>

            {#if selectedPrioritization == "population_density"}
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
          {/snippet}
        </Popup>
      </FillLayer>
    </GeoJSON>

    <ModalFilterLayer interactive={false} />
  {/snippet}
</SplitComponent>

<style>
  li.highlighted {
    background-color: rgba(72, 96, 202, 0.15);
  }
</style>
