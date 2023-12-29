<script lang="ts">
  import { LTN } from "backend";
  import type { Feature, LineString, Polygon } from "geojson";
  import type { Map, MapMouseEvent } from "maplibre-gl";
  import { onDestroy } from "svelte";
  import { Popup } from "svelte-maplibre";
  import { PropertiesTable } from "./common";
  import FreehandLine from "./FreehandLine.svelte";
  import RenderNeighbourhood from "./RenderNeighbourhood.svelte";
  import SplitComponent from "./SplitComponent.svelte";
  import { app, mode } from "./stores";

  // Caller is responsible for doing app.setNeighbourhood

  export let map: Map;
  export let showBasemap: boolean;

  let addingFilter = false;
  let addingMultipleFilters = false;
  let undoLength = 0;
  let redoLength = 0;
  let boundary: Feature<Polygon> | null;

  let gjInput;
  render($app.renderNeighbourhood());

  function render(gjString) {
    gjInput = JSON.parse(gjString);
    boundary = gjInput.features.find((f) => f.properties.kind == "boundary");

    undoLength = gjInput.undo_length;
    redoLength = gjInput.redo_length;
  }

  $: if (addingFilter) {
    map.on("click", onClick);
    map.style.cursor = "crosshair";
  }
  onDestroy(() => {
    stopAddingFilter();
    // TODO Then we can't "nest" ViewShortcuts beneath this
    //$app.unsetNeighbourhood();
  });
  function onClick(e: MapMouseEvent) {
    render($app.addModalFilter(e.lngLat));
    stopAddingFilter();
  }
  function stopAddingFilter() {
    addingFilter = false;
    map.off("click", onClick);
    map.style.cursor = "inherit";
  }

  function deleteFilter(f: Feature) {
    if (f.properties.kind == "modal_filter") {
      render($app.deleteModalFilter(f.properties.road));
    }
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
    render($app.undo());
  }
  function redo() {
    render($app.redo());
  }

  function reset() {
    $mode = {
      mode: "network",
    };
  }

  function gotFreehandLine(e: CustomEvent<Feature<LineString> | null>) {
    let f = e.detail;
    if (f) {
      render($app.addManyModalFilters(f));
    }

    addingMultipleFilters = false;
  }
</script>

<svelte:window on:keydown={onKeyDown} />

<SplitComponent>
  <div slot="sidebar">
    <div><button on:click={reset}>Reset</button></div>
    <div>
      <button
        on:click={() => ($mode = { mode: "set-boundary", existing: boundary })}
        >Edit boundary</button
      >
    </div>
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
    </div>
    <div>
      <button
        on:click={() => ($mode = { mode: "view-shortcuts", prevMode: $mode })}
        >View shortcuts</button
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
  </div>

  <div slot="map">
    <RenderNeighbourhood
      {gjInput}
      {showBasemap}
      onClickLine={(f) => window.open(f.properties.way, "_blank")}
      onClickCircle={deleteFilter}
    >
      <div slot="line-popup">
        <Popup openOn="hover" let:data>
          <PropertiesTable properties={data.properties} />
        </Popup>
      </div>
      <div slot="circle-popup">
        <Popup openOn="hover" let:data>
          <PropertiesTable properties={data.properties} />
        </Popup>
      </div>
    </RenderNeighbourhood>
    {#if addingMultipleFilters}
      <FreehandLine {map} on:done={gotFreehandLine} />
    {/if}
  </div>
</SplitComponent>
