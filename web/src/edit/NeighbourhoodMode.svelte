<script lang="ts">
  import type {
    Feature,
    FeatureCollection,
    LineString,
    Polygon,
  } from "geojson";
  import type { MapMouseEvent } from "maplibre-gl";
  import { onDestroy } from "svelte";
  import { type LayerClickInfo } from "svelte-maplibre";
  import { notNull, Popup, Link } from "../common";
  import ManageSavefiles from "../ManageSavefiles.svelte";
  import RenderNeighbourhood from "../RenderNeighbourhood.svelte";
  import SplitComponent from "../SplitComponent.svelte";
  import { app, map, mode, mutationCounter, filterType } from "../stores";
  import ChangeModalFilter from "./ChangeModalFilter.svelte";
  import FreehandLine from "./FreehandLine.svelte";
  import ModalFilterLayer from "../ModalFilterLayer.svelte";

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
  let boundary: Feature<Polygon> | null;

  let gjInput: FeatureCollection;
  $: rerender($mutationCounter);

  $map!.on("click", onClick);
  onDestroy(() => {
    $map!.off("click", onClick);
    $map!.doubleClickZoom.enable();
  });

  function rerender(_x: number) {
    gjInput = JSON.parse($app!.renderNeighbourhood());
    boundary = gjInput.features.find(
      (f) => f.properties!.kind == "boundary",
    )! as Feature<Polygon>;

    // @ts-ignore These foreign members exist
    undoLength = gjInput.undo_length;
    // @ts-ignore These foreign members exist
    redoLength = gjInput.redo_length;
  }

  function onClick(e: MapMouseEvent) {
    if (action == "filter") {
      $app!.addModalFilter(e.lngLat, $filterType);
      $mutationCounter++;
    }
  }

  function onClickLine(f: Feature) {
    if (action == "oneway") {
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
          <Link on:click={() => ($mode = { mode: "title" })}>
            Choose project
          </Link>
        </li>
        <li>
          <Link on:click={() => ($mode = { mode: "network" })}>
            Pick neighbourhood
          </Link>
        </li>
        <li>Editing</li>
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
          <Link
            on:click={() =>
              ($mode = {
                mode: "set-boundary",
                name: notNull(notNull(boundary).properties).name,
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
      Editing neighbourhood <u>{notNull(notNull(boundary).properties).name}</u>
    </p>

    <p>
      Now that you've defined a neighbourhood boundary, you can see the possible
      shortcuts that vehicles are currently able to take through it. You can add
      a new modal filter to try and solve this. The colored "cell" areas show
      what's reachable for drivers without leaving the boundary you've drawn.
    </p>

    <hr />

    <div style="display: flex; justify-content: space-between;">
      <button
        on:click={() => (action = "filter")}
        disabled={action == "filter"}
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
      >
        Add many modal filters along line
      </button>
      <button
        on:click={() => (action = "oneway")}
        disabled={action == "oneway"}
      >
        Reverse directions
      </button>
    </div>

    <button on:click={() => (settingFilterType = true)}>
      Change modal filter type
    </button>

    <div style="display: flex; justify-content: space-between;">
      <button disabled={undoLength == 0} on:click={undo}>
        {#if undoLength == 0}
          Undo
        {:else}
          Undo ({undoLength})
        {/if}
      </button>
      <button disabled={redoLength == 0} on:click={redo}>
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

    <hr />

    <hr />
    <ManageSavefiles />
  </div>

  <div slot="map">
    <RenderNeighbourhood
      {gjInput}
      interactive={action == "oneway"}
      {onClickLine}
    >
      <div slot="line-popup">
        <Popup openOn="hover" let:props>
          <p>
            {props.shortcuts} shortcuts through {props.name ?? "unnamed road"}
          </p>
          {#if action == "oneway"}
            <p>Click to change direction</p>
          {/if}
        </Popup>
      </div>
    </RenderNeighbourhood>
    <ModalFilterLayer on:click={deleteFilter}>
      <Popup openOn="hover">Click to delete</Popup>
    </ModalFilterLayer>
    {#if action == "freehand-filters"}
      <FreehandLine map={notNull($map)} on:done={gotFreehandLine} />
    {/if}
  </div>
</SplitComponent>
