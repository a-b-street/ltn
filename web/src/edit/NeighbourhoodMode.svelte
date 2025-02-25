<script lang="ts">
  import type { Feature, FeatureCollection, LineString } from "geojson";
  import type { LngLat, MapMouseEvent } from "maplibre-gl";
  import { onDestroy } from "svelte";
  import {
    Control,
    GeoJSON,
    hoverStateFilter,
    LineLayer,
    MapEvents,
    type LayerClickInfo,
  } from "svelte-maplibre";
  import { notNull, SequentialLegend } from "svelte-utils";
  import { emptyGeojson, Popup } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import AnimatePaths from "../AnimatePaths.svelte";
  import { HelpButton, layerId, Link, roadLineWidth } from "../common";
  import { speedColorScale, speedLimits } from "../common/colors";
  import type { Intersection } from "../common/Intersection";
  import {
    CellLayer,
    HighlightBoundaryLayer,
    InteriorRoadLayer,
    ModalFilterLayer,
    OneWayLayer,
    RenderNeighbourhood,
  } from "../layers";
  import EditableIntersectionLayer from "../layers/EditableIntersectionLayer.svelte";
  import {
    animateShortcuts,
    autosave,
    backend,
    editPerimeterRoads,
    filterType,
    map,
    mode,
    mutationCounter,
    roadStyle,
  } from "../stores";
  import type {
    NeighbourhoodBoundaryFeature,
    RenderNeighbourhoodOutput,
  } from "../wasm";
  import ChangeModalFilter from "./ChangeModalFilter.svelte";
  import FreehandLine from "./FreehandLine.svelte";

  // Caller is responsible for doing backend.setCurrentNeighbourhood

  type Action =
    | { kind: "filter" }
    | { kind: "freehand-filters" }
    | { kind: "oneway" }
    | {
        kind: "turn_restriction";
        from_road_id: number | null;
        from_road_name: string;
        possible_targets: FeatureCollection<
          LineString,
          { road: number; name: string }
        >;
      };
  function startTurnRestrictionAction(): Action {
    return {
      kind: "turn_restriction",
      from_road_id: null,
      from_road_name: "",
      possible_targets: emptyGeojson() as FeatureCollection<
        LineString,
        { road: number; name: string }
      >,
    };
  }
  let action: Action = { kind: "filter" };

  $: if (action.kind == "oneway") {
    $map!.doubleClickZoom.disable();
  } else {
    $map!.doubleClickZoom.enable();
  }

  let settingFilterType = false;
  let undoLength = 0;
  let redoLength = 0;
  let boundary: NeighbourhoodBoundaryFeature | null;

  let gj: RenderNeighbourhoodOutput;
  let allShortcuts = $backend!.getAllShortcuts();
  $: rerender($mutationCounter);

  $: numDisconnectedCells = gj.features.filter(
    (f) =>
      f.properties.kind == "cell" && f.properties.cell_color == "disconnected",
  ).length;

  onDestroy(() => {
    $map!.doubleClickZoom.enable();
  });

  function rerender(_x: number) {
    gj = $backend!.renderNeighbourhood();
    // @ts-expect-error TS can't figure out that we're narrowing the case here
    boundary = gj.features.find((f) => f.properties.kind == "boundary")!;

    undoLength = gj.undo_length;
    redoLength = gj.redo_length;

    allShortcuts = $backend!.getAllShortcuts();

    autosave();
  }

  function recalculateNeighbourhoodDefinition() {
    $backend!.setCurrentNeighbourhood(
      boundary!.properties.name,
      $editPerimeterRoads,
    );
    $mutationCounter++;
  }

  function onClickLine(f: Feature, pt: LngLat) {
    if (action.kind == "filter") {
      $backend!.addModalFilter(pt, $filterType);
      $mutationCounter++;
    } else if (action.kind == "oneway") {
      $backend!.toggleTravelFlow(f.properties!.road);
      $mutationCounter++;
    } else if (action.kind == "turn_restriction") {
      action.from_road_id = f.properties!.road;
      action.from_road_name = f.properties!.name || "unnamed road";
      action.possible_targets = $backend!.getTurnRestrictionTargets(
        f.properties!.road,
      );
      // TODO The blue hover state gets stuck
    }
  }

  function onMapClick(e: CustomEvent<MapMouseEvent>) {
    if (action.kind != "turn_restriction") {
      return;
    }

    // If we click a blank area, reset some state
    if (
      $map!.queryRenderedFeatures(e.detail.point, {
        layers: ["interior-roads"],
      }).length > 0
    ) {
      return;
    }

    action = startTurnRestrictionAction();
  }

  function createTurnRestriction(e: CustomEvent<LayerClickInfo>) {
    if (action.kind == "turn_restriction" && action.from_road_id) {
      let to = e.detail.features[0].properties!.road;
      window.alert(`TODO: create TR from ${action.from_road_id} to ${to}`);
    }
    action = { kind: "filter" };
  }

  function deleteFilter(e: CustomEvent<LayerClickInfo>) {
    let f = e.detail.features[0];
    $backend!.deleteModalFilter(f.properties!.road);
    $mutationCounter++;
  }

  function onClickIntersection(intersection: Intersection) {
    if (action.kind != "filter") {
      console.assert(
        false,
        `this shouldn't happen - intersections should only be clickable when in 'filter' mode, not ${action.kind}`,
      );
      return;
    }

    if (intersection.hasRotatedFilter) {
      $backend!.deleteDiagonalFilter(intersection);
      $mutationCounter++;
    } else if (intersection.filter) {
      $backend!.rotateDiagonalFilter(intersection);
      $mutationCounter++;
    } else {
      $backend!.addDiagonalFilter(intersection);
      $mutationCounter++;
    }
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
      action = { kind: "filter" };
    }
    if (e.key == "2") {
      action = { kind: "freehand-filters" };
    }
    if (e.key == "3") {
      action = { kind: "oneway" };
    }
    if (e.key == "4") {
      action = startTurnRestrictionAction();
    }
  }
  function undo() {
    $backend!.undo();
    $mutationCounter++;
  }
  function redo() {
    $backend!.redo();
    $mutationCounter++;
  }

  function gotFreehandLine(e: CustomEvent<Feature<LineString> | null>) {
    let f = e.detail;
    if (f) {
      $backend!.addManyModalFilters(f, $filterType);
      $mutationCounter++;
    }

    action = { kind: "filter" };
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
          <Link on:click={() => ($mode = { mode: "pick-neighbourhood" })}>
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
          <Link on:click={() => ($mode = { mode: "debug-neighbourhood" })}>
            Debug
          </Link>
        </li>
      </ul>
    </nav>
  </div>
  <div slot="sidebar">
    <p>
      Editing neighbourhood <u>{notNull(boundary).properties.name}</u>
      , with an area of {notNull(boundary).properties.area_km2.toFixed(1)} km²
    </p>

    {#if numDisconnectedCells > 0}
      <mark>
        Some parts of the neighbourhood aren't reachable by drivers, shown in
        red
      </mark>
    {/if}

    <button class="outline" on:click={() => (settingFilterType = true)}>
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

    <div style="border: 1px solid black; padding: 4px">
      <label>
        Draw roads:
        <select bind:value={$roadStyle}>
          <option value="shortcuts">Worst shortcuts</option>
          <option value="cells">Cell</option>
          <option value="edits">Edits (either filter or direction)</option>
          <option value="speeds">Speed limit</option>
        </select>
      </label>
      {#if $roadStyle == "speeds"}
        <SequentialLegend colorScale={speedColorScale} limits={speedLimits} />
      {/if}
    </div>

    <div style="display: flex; justify-content: space-between;">
      <button
        class="outline"
        disabled={undoLength == 0}
        on:click={undo}
        data-tooltip="Ctrl+Z"
      >
        {#if undoLength == 0}
          Undo
        {:else}
          Undo ({undoLength})
        {/if}
      </button>
      <button
        class="outline"
        disabled={redoLength == 0}
        on:click={redo}
        data-tooltip="Ctrl+Y"
      >
        {#if redoLength == 0}
          Redo
        {:else}
          Redo ({redoLength})
        {/if}
      </button>
    </div>

    <ChangeModalFilter bind:show={settingFilterType} />
  </div>

  <div slot="map">
    <MapEvents on:click={onMapClick} />

    <Control position="bottom-right">
      <div class="pico">
        <button
          on:click={() => (action = { kind: "filter" })}
          disabled={action.kind == "filter"}
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
          on:click={() => (action = { kind: "freehand-filters" })}
          disabled={action.kind == "freehand-filters"}
          data-tooltip="hotkey 2"
        >
          Add many modal filters along line
        </button>
        <button
          on:click={() => (action = { kind: "oneway" })}
          disabled={action.kind == "oneway"}
          data-tooltip="hotkey 3"
        >
          Reverse directions
        </button>
        <button
          on:click={() => (action = startTurnRestrictionAction())}
          disabled={action.kind == "turn_restriction"}
          data-tooltip="hotkey 4"
        >
          Restrict turns
        </button>
      </div>
    </Control>

    <RenderNeighbourhood input={gj}>
      <HighlightBoundaryLayer />
      <CellLayer />
      <OneWayLayer />

      <InteriorRoadLayer
        interactive={action.kind == "filter" ||
          action.kind == "oneway" ||
          (action.kind == "turn_restriction" && action.from_road_id == null)}
        {onClickLine}
      >
        <div slot="line-popup">
          <Popup openOn="hover" let:props>
            <p>
              {props.shortcuts} shortcuts through {props.name ?? "unnamed road"}
              ({Math.round(props.speed_mph)} mph)
            </p>
            {#if action.kind == "filter"}
              <div>
                <img
                  src={`${import.meta.env.BASE_URL}/filters/${$filterType}_icon.gif`}
                  width="20"
                  alt="Add modal filter"
                />
                Click to add modal filter
              </div>
            {:else if action.kind == "oneway"}
              <p>Click to change direction</p>
            {:else if action.kind == "turn_restriction"}
              <p>Click to create a turn restriction from here</p>
            {/if}
          </Popup>
        </div>
      </InteriorRoadLayer>
      <EditableIntersectionLayer
        interactive={action.kind == "filter"}
        neighbourhood={gj}
        {onClickIntersection}
      />
    </RenderNeighbourhood>

    {#if $animateShortcuts}
      <AnimatePaths paths={allShortcuts} />
    {/if}

    <ModalFilterLayer on:click={deleteFilter}>
      <Popup openOn="hover">Click to delete</Popup>
    </ModalFilterLayer>

    {#if action.kind == "freehand-filters"}
      <FreehandLine map={notNull($map)} on:done={gotFreehandLine} />
    {/if}

    {#if action.kind == "turn_restriction"}
      <GeoJSON data={action.possible_targets || emptyGeojson()} generateId>
        <LineLayer
          {...layerId("turn-restriction-targets")}
          manageHoverState
          paint={{
            "line-color": "yellow",
            "line-opacity": hoverStateFilter(0.5, 1.0),
            "line-width": roadLineWidth(1),
          }}
          on:click={createTurnRestriction}
        >
          <Popup openOn="hover" let:props>
            Create a turn restriction from {action.from_road_name} to {props.name ||
              "unnamed road"}
          </Popup>
        </LineLayer>
      </GeoJSON>
    {/if}
  </div>
</SplitComponent>
