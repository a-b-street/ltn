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
  import { notNull, Popup } from "../common";
  import ManageSavefiles from "../ManageSavefiles.svelte";
  import RenderNeighbourhood from "../RenderNeighbourhood.svelte";
  import SplitComponent from "../SplitComponent.svelte";
  import { app, map, mode, mutationCounter, filterType } from "../stores";
  import ChangeModalFilter from "./ChangeModalFilter.svelte";
  import FreehandLine from "./FreehandLine.svelte";
  import ModalFilterLayer from "../ModalFilterLayer.svelte";

  // Caller is responsible for doing app.setCurrentNeighbourhood

  let addingFilter = false;
  let addingMultipleFilters = false;
  let settingFilterType = false;
  let undoLength = 0;
  let redoLength = 0;
  let boundary: Feature<Polygon> | null;

  let gjInput: FeatureCollection;
  $: rerender($mutationCounter);

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

  $: if (addingFilter) {
    $map!.on("click", onClick);
    // TODO Still doesn't last long
    $map!.getCanvas().style.cursor = "crosshair";
  }
  onDestroy(() => {
    stopAddingFilter();
  });
  function onClick(e: MapMouseEvent) {
    $app!.addModalFilter(e.lngLat, $filterType);
    $mutationCounter++;
    stopAddingFilter();
  }
  function stopAddingFilter() {
    addingFilter = false;
    $map!.off("click", onClick);
    $map!.getCanvas().style.cursor = "inherit";
  }

  function deleteFilter(e: CustomEvent<LayerClickInfo>) {
    let f = e.detail.features[0];
    $app!.deleteModalFilter(f.properties!.road);
    $mutationCounter++;
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key == "a" && !addingFilter && !addingMultipleFilters) {
      addingFilter = true;
    }
    if (e.key == "z" && e.ctrlKey) {
      undo();
    }
    if (e.key == "y" && e.ctrlKey) {
      redo();
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

    addingMultipleFilters = false;
  }
</script>

<svelte:window on:keydown={onKeyDown} />

<SplitComponent>
  <div slot="top" style="display: flex; justify-content: space-between;">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <a href="#" on:click={() => ($mode = { mode: "title" })}>
            Choose study area
          </a>
        </li>
        <li>
          <a href="#" on:click={() => ($mode = { mode: "network" })}>
            Pick neighbourhood
          </a>
        </li>
        <li>Editing modal filters</li>
      </ul>
    </nav>
    <nav>
      <ul>
        <li>
          <a href="#" on:click={() => ($mode = { mode: "view-shortcuts" })}>
            View shortcuts
          </a>
        </li>
        <li>
          <a
            href="#"
            on:click={() =>
              ($mode = { mode: "route", prevMode: "neighbourhood" })}
          >
            Route
          </a>
        </li>
        <li>
          <a
            href="#"
            on:click={() =>
              ($mode = {
                mode: "set-boundary",
                name: notNull(notNull(boundary).properties).name,
                existing: boundary,
              })}
          >
            Change this boundary
          </a>
        </li>
        <li>
          <a href="#" on:click={() => ($mode = { mode: "debug" })}>Debug</a>
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

    <button
      on:click={() => (addingFilter = true)}
      disabled={addingFilter || addingMultipleFilters}
    >
      <img
        src={`${import.meta.env.BASE_URL}/filters/${$filterType}_icon.gif`}
        width="30"
        alt="Add a modal filter"
      />
      Add a modal filter
    </button>
    <button
      on:click={() => (addingMultipleFilters = true)}
      disabled={addingFilter || addingMultipleFilters}
    >
      Add many modal filters along line
    </button>
    <button
      on:click={() => (settingFilterType = true)}
      disabled={addingFilter || addingMultipleFilters}
    >
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
      interactive={!addingFilter && !addingMultipleFilters}
    >
      <div slot="line-popup">
        <Popup openOn="hover" let:props>
          <p>
            {props.shortcuts} shortcuts through {props.name ?? "unnamed road"}
          </p>
        </Popup>
      </div>
    </RenderNeighbourhood>
    <ModalFilterLayer on:click={deleteFilter}>
      <Popup openOn="hover">Click to delete</Popup>
    </ModalFilterLayer>
    {#if addingMultipleFilters}
      <FreehandLine map={notNull($map)} on:done={gotFreehandLine} />
    {/if}
  </div>
</SplitComponent>
