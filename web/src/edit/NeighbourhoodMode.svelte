<script lang="ts">
  import type { Feature, LineString, Polygon } from "geojson";
  import type { LngLat } from "maplibre-gl";
  import type { Waypoint } from "route-snapper-ts";
  import { onDestroy } from "svelte";
  import { type LayerClickInfo } from "svelte-maplibre";
  import { notNull } from "svelte-utils";
  import { Popup } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import AnimatePaths from "../AnimatePaths.svelte";
  import { HelpButton, Link } from "../common";
  import ModalFilterLayer from "../ModalFilterLayer.svelte";
  import RenderNeighbourhood from "../RenderNeighbourhood.svelte";
  import {
    animateShortcuts,
    app,
    autosave,
    editPerimeterRoads,
    filterType,
    map,
    mode,
    mutationCounter,
  } from "../stores";
  import type { RenderNeighbourhoodOutput } from "../wasm";
  import ChangeModalFilter from "./ChangeModalFilter.svelte";
  import FreehandLine from "./FreehandLine.svelte";

  // Caller is responsible for doing app.setCurrentNeighbourhood

  type Action = "filter" | "freehand-filters" | "oneway";
  let action: Action = "filter";

  $: if (action == "oneway") {
    $map!.doubleClickZoom.disable();
  } else {
    $map!.doubleClickZoom.enable();
  }

  let settingFilterType = false;
  let undoLength = 0;
  let redoLength = 0;
  let boundary: Feature<
    Polygon,
    { name: string; waypoints: Waypoint[] }
  > | null;

  let gjInput: RenderNeighbourhoodOutput;
  let allShortcuts = JSON.parse($app!.getAllShortcuts());
  $: rerender($mutationCounter);

  $: numDisconnectedCells = gjInput.features.filter(
    (f) =>
      f.properties.kind == "cell" && f.properties.cell_color == "disconnected",
  ).length;

  onDestroy(() => {
    $map!.doubleClickZoom.enable();
  });

  function rerender(_x: number) {
    gjInput = JSON.parse($app!.renderNeighbourhood());
    // @ts-expect-error TS can't figure out that we're narrowing the case here
    boundary = gjInput.features.find((f) => f.properties.kind == "boundary")!;

    undoLength = gjInput.undo_length;
    redoLength = gjInput.redo_length;

    allShortcuts = JSON.parse($app!.getAllShortcuts());

    autosave();
  }

  function recalculateNeighbourhoodDefinition() {
    $app!.setCurrentNeighbourhood(
      boundary!.properties.name,
      $editPerimeterRoads,
    );
    $mutationCounter++;
  }

  function onClickLine(f: Feature, pt: LngLat) {
    if (action == "filter") {
      $app!.addModalFilter(pt, $filterType);
      $mutationCounter++;
    } else if (action == "oneway") {
      $app!.toggleDirection(f.properties!.road);
      $mutationCounter++;
    }
  }

  function deleteFilter(e: CustomEvent<LayerClickInfo>) {
    let f = e.detail.features[0];
    $app!.deleteModalFilter(f.properties!.road);
    $mutationCounter++;
  }

  function onKeyDown(e: KeyboardEvent) {
    // Ignore keypresses if we're not focused on the map
    if ((e.target as HTMLElement).tagName == "INPUT") {
      return;
    }
    if (e.ctrlKey && e.key == "z") {
      undo();
    }
    if (e.ctrlKey && e.key == "y") {
      redo();
    }
    if (e.key == "1") {
      action = "filter";
    }
    if (e.key == "2") {
      action = "freehand-filters";
    }
    if (e.key == "3") {
      action = "oneway";
    }
  }
  function undo() {
    $app!.undo();
    $mutationCounter++;
  }
  function redo() {
    $app!.redo();
    $mutationCounter++;
  }

  function gotFreehandLine(e: CustomEvent<Feature<LineString> | null>) {
    let f = e.detail;
    if (f) {
      $app!.addManyModalFilters(f, $filterType);
      $mutationCounter++;
    }

    action = "filter";
  }
</script>

<svelte:window on:keydown={onKeyDown} />

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
          <Link on:click={() => ($mode = { mode: "network" })}>
            Pick neighbourhood
          </Link>
        </li>
        <li>
          Editing
          <HelpButton>
            <p>
              Now that you've defined a neighbourhood boundary, you can see the
              possible shortcuts that vehicles are currently able to take
              through it. You can add a new modal filter to try and solve this.
              The colored "cell" areas show what's reachable for drivers without
              leaving the boundary you've drawn.
            </p>
          </HelpButton>
        </li>
      </ul>
    </nav>
    <nav>
      <ul>
        <li>
          <Link on:click={() => ($mode = { mode: "view-shortcuts" })}>
            View shortcuts
          </Link>
        </li>
        <li>
          <Link
            on:click={() =>
              ($mode = { mode: "route", prevMode: "neighbourhood" })}
          >
            Route
          </Link>
        </li>
        <li>
          <Link on:click={() => ($mode = { mode: "impact-one-destination" })}>
            Impact routing to one destination
          </Link>
        </li>
        <li>
          <Link
            on:click={() =>
              ($mode = {
                mode: "set-boundary",
                name: notNull(boundary).properties.name,
                existing: boundary,
              })}
          >
            Change this boundary
          </Link>
        </li>
        <li>
          <Link on:click={() => ($mode = { mode: "debug" })}>Debug</Link>
        </li>
      </ul>
    </nav>
  </div>
  <div slot="sidebar">
    <p>
      Editing neighbourhood <u>{notNull(boundary).properties.name}</u>
      , with an area of {gjInput.area_km2.toFixed(1)} km²
    </p>

    {#if numDisconnectedCells > 0}
      <mark>
        Some parts of the neighbourhood aren't reachable by drivers, shown in
        red
      </mark>
    {/if}

    <div style="display: flex; justify-content: space-between;">
      <button
        on:click={() => (action = "filter")}
        disabled={action == "filter"}
        data-tooltip="hotkey 1"
      >
        <img
          src={`${import.meta.env.BASE_URL}/filters/${$filterType}_icon.gif`}
          width="30"
          alt="Add a modal filter"
        />
        Add a modal filter
      </button>
      <button
        on:click={() => (action = "freehand-filters")}
        disabled={action == "freehand-filters"}
        data-tooltip="hotkey 2"
      >
        Add many modal filters along line
      </button>
      <button
        on:click={() => (action = "oneway")}
        disabled={action == "oneway"}
        data-tooltip="hotkey 3"
      >
        Reverse directions
      </button>
    </div>

    <button on:click={() => (settingFilterType = true)}>
      Change modal filter type
    </button>

    <label>
      <input type="checkbox" bind:checked={$animateShortcuts} />
      Animate shortcuts
    </label>

    <label>
      <input
        type="checkbox"
        bind:checked={$editPerimeterRoads}
        on:change={recalculateNeighbourhoodDefinition}
      />
      Include perimeter roads
    </label>

    <div style="display: flex; justify-content: space-between;">
      <button disabled={undoLength == 0} on:click={undo} data-tooltip="Ctrl+Z">
        {#if undoLength == 0}
          Undo
        {:else}
          Undo ({undoLength})
        {/if}
      </button>
      <button disabled={redoLength == 0} on:click={redo} data-tooltip="Ctrl+Y">
        {#if redoLength == 0}
          Redo
        {:else}
          Redo ({redoLength})
        {/if}
      </button>
    </div>

    {#if settingFilterType}
      <ChangeModalFilter on:close={() => (settingFilterType = false)} />
    {/if}
  </div>

  <div slot="map">
    <RenderNeighbourhood
      {gjInput}
      interactive={action == "filter" || action == "oneway"}
      {onClickLine}
    >
      <div slot="line-popup">
        <Popup openOn="hover" let:props>
          <p>
            {props.shortcuts} shortcuts through {props.name ?? "unnamed road"}
          </p>
          {#if action == "filter"}
            <div>
              <img
                src={`${import.meta.env.BASE_URL}/filters/${$filterType}_icon.gif`}
                width="20"
                alt="Add modal filter"
              />
              Click to add modal filter
            </div>
          {:else}
            <p>Click to change direction</p>
          {/if}
        </Popup>
      </div>
    </RenderNeighbourhood>
    {#if $animateShortcuts}
      <AnimatePaths paths={allShortcuts} />
    {/if}
    <ModalFilterLayer on:click={deleteFilter}>
      <Popup openOn="hover">Click to delete</Popup>
    </ModalFilterLayer>
    {#if action == "freehand-filters"}
      <FreehandLine map={notNull($map)} on:done={gotFreehandLine} />
    {/if}
  </div>
</SplitComponent>
