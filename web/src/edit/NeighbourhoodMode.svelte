<script lang="ts">
  import type {
    Feature,
    FeatureCollection,
    LineString,
    Polygon,
  } from "geojson";
  import type { MapMouseEvent } from "maplibre-gl";
  import { onDestroy } from "svelte";
  import { GeoJSON, Popup, SymbolLayer } from "svelte-maplibre";
  import { notNull } from "../common";
  import ManageSavefiles from "../ManageSavefiles.svelte";
  import RenderNeighbourhood from "../RenderNeighbourhood.svelte";
  import SplitComponent from "../SplitComponent.svelte";
  import { app, map, mode, mutationCounter } from "../stores";
  import ChangeModalFilter from "./ChangeModalFilter.svelte";
  import FreehandLine from "./FreehandLine.svelte";

  // Caller is responsible for doing app.setCurrentNeighbourhood

  let filterType = "walk_cycle_only";
  let addingFilter = false;
  let addingMultipleFilters = false;
  let settingFilterType = false;
  let undoLength = 0;
  let redoLength = 0;
  let boundary: Feature<Polygon> | null;

  let gjInput: FeatureCollection;
  let modalFilterGj: FeatureCollection;
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

    modalFilterGj = JSON.parse($app!.renderModalFilters());
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
    $app!.addModalFilter(e.lngLat, filterType);
    $mutationCounter++;
    stopAddingFilter();
  }
  function stopAddingFilter() {
    addingFilter = false;
    $map!.off("click", onClick);
    $map!.getCanvas().style.cursor = "inherit";
  }

  function deleteFilter(e) {
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

  function pickNewNeighbourhood() {
    $mode = {
      mode: "network",
    };
  }

  function gotFreehandLine(e: CustomEvent<Feature<LineString> | null>) {
    let f = e.detail;
    if (f) {
      $app!.addManyModalFilters(f, filterType);
      $mutationCounter++;
    }

    addingMultipleFilters = false;
  }
</script>

<svelte:window on:keydown={onKeyDown} />

<SplitComponent>
  <div slot="sidebar">
    <h1>
      Editing modal filters in {notNull(notNull(boundary).properties).name}
    </h1>
    <p>
      Now that you've defined a neighbourhood boundary, you can see the possible
      shortcuts that vehicles are currently able to take through it. You can add
      a new modal filter to try and solve this. The colored "cell" areas show
      what's reachable for drivers without leaving the boundary you've drawn.
    </p>

    <div>
      <button on:click={() => ($mode = { mode: "title" })}
        >Start over and change your study area</button
      >
    </div>
    <div>
      <button on:click={pickNewNeighbourhood}
        >Pick a different neighbourhood boundary</button
      >
    </div>
    <div>
      <button
        on:click={() =>
          ($mode = {
            mode: "set-boundary",
            name: notNull(notNull(boundary).properties).name,
            existing: boundary,
          })}>Change this neighbourhood boundary</button
      >
    </div>

    <hr />

    <div>
      <button
        on:click={() => (addingFilter = true)}
        disabled={addingFilter || addingMultipleFilters}
        >Add a modal filter</button
      >
      <button
        on:click={() => (addingMultipleFilters = true)}
        disabled={addingFilter || addingMultipleFilters}
        >Add many modal filters along line</button
      >
      <button
        on:click={() => (settingFilterType = true)}
        disabled={addingFilter || addingMultipleFilters}
        >Change modal filter type</button
      >
    </div>
    <div>
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
      <ChangeModalFilter
        bind:filterType
        on:close={() => (settingFilterType = false)}
      />
    {/if}

    <hr />

    <div>
      <button on:click={() => ($mode = { mode: "view-shortcuts" })}
        >View shortcuts</button
      >
    </div>
    <div>
      <button on:click={() => ($mode = { mode: "route" })}>Route</button>
    </div>
    <div>
      <button on:click={() => ($mode = { mode: "debug" })}>Debug</button>
    </div>

    <hr />
    <ManageSavefiles />
  </div>

  <div slot="map">
    <RenderNeighbourhood
      {gjInput}
      interactive={!addingFilter && !addingMultipleFilters}
    >
      <div slot="line-popup">
        <Popup openOn="hover" let:data
          ><p>{notNull(data).properties.shortcuts} shortcuts</p></Popup
        >
      </div>
    </RenderNeighbourhood>
    <GeoJSON data={modalFilterGj} generateId>
      <SymbolLayer
        layout={{
          "icon-image": ["get", "filter_kind"],
          "icon-allow-overlap": true,
          "icon-size": 0.1,
        }}
        on:click={deleteFilter}
      >
        <Popup openOn="hover">Click to delete</Popup>
      </SymbolLayer>
    </GeoJSON>
    {#if addingMultipleFilters}
      <FreehandLine map={notNull($map)} on:done={gotFreehandLine} />
    {/if}
  </div>
</SplitComponent>
