<script lang="ts">
  import type { Feature, FeatureCollection, LineString } from "geojson";
  import type { LngLat, MapMouseEvent } from "maplibre-gl";
  import { onDestroy } from "svelte";
  import {
    GeoJSON,
    hoverStateFilter,
    LineLayer,
    MapEvents,
    type LayerClickInfo,
  } from "svelte-maplibre";
  import { notNull } from "svelte-utils";
  import { emptyGeojson, Popup } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import onewayArrowUrl from "../../assets/one_way_left.svg?url";
  import AnimatePaths from "../AnimatePaths.svelte";
  import {
    HelpButton,
    layerId,
    Link,
    roadLineWidth,
    SequentialLegend,
  } from "../common";
  import { speedColorScale, speedLimits } from "../common/colors";
  import type { Intersection } from "../common/Intersection";
  import NeighbourhoodBoundarySummary from "../common/NeighbourhoodBoundarySummary.svelte";
  import {
    CellLayer,
    HighlightBoundaryLayer,
    ModalFilterLayer,
    NeighbourhoodRoadLayer,
    OneWayLayer,
    RenderNeighbourhood,
  } from "../layers";
  import EditableIntersectionLayer from "../layers/EditableIntersectionLayer.svelte";
  import {
    animateShortcuts,
    backend,
    devMode,
    filterType,
    map,
    mode,
    mutationCounter,
    returnToChooseProject,
    roadStyle,
    saveCurrentProject,
  } from "../stores";
  import type {
    NeighbourhoodBoundaryFeature,
    RenderNeighbourhoodOutput,
  } from "../wasm";
  import ChangeFilterModal from "./ChangeFilterModal.svelte";
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
      }
    | { kind: "main-roads" };
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

  $: if (action.kind == "oneway" || action.kind == "main-roads") {
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

    saveCurrentProject();
  }

  function onClickLine(f: Feature, pt: LngLat) {
    if (action.kind == "filter") {
      $backend!.addModalFilter(pt, $filterType);
      $mutationCounter++;
    } else if (action.kind == "oneway") {
      $backend!.toggleTravelFlow(f.properties!.road);
      $mutationCounter++;
    } else if (action.kind == "main-roads") {
      $backend!.toggleMainRoad(f.properties!.road);
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

    // If we click a blank area, reset some state. Not sure why, but clicking
    // layers doesn't always prevent a click on the map itself.
    if (
      $map!.queryRenderedFeatures(e.detail.point, {
        layers: ["interior-roads", "turn-restriction-targets"],
      }).length > 0
    ) {
      return;
    }

    action = startTurnRestrictionAction();
  }

  function createTurnRestriction(e: CustomEvent<LayerClickInfo>) {
    if (action.kind == "turn_restriction" && action.from_road_id != null) {
      let to = e.detail.features[0].properties!.road;
      $backend!.addTurnRestriction(action.from_road_id, to);
      $mutationCounter++;
    }
    action = startTurnRestrictionAction();
  }

  function deleteModalFilter(e: CustomEvent<LayerClickInfo>) {
    let f = e.detail.features[0];
    $backend!.deleteModalFilter(f.properties!.road);
    $mutationCounter++;
  }

  function deleteTurnRestriction(e: CustomEvent<LayerClickInfo>) {
    let f = e.detail.features[0];
    $backend!.deleteTurnRestriction(
      f.properties!.intersection,
      f.properties!.from_road,
      f.properties!.to_road,
    );
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
    if (e.key == "5") {
      action = { kind: "main-roads" };
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
          <Link on:click={returnToChooseProject}>Choose project</Link>
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
                existing: notNull(boundary),
              })}
          >
            Change this boundary
          </Link>
        </li>
        {#if $devMode}
          <li>
            <Link on:click={() => ($mode = { mode: "debug-neighbourhood" })}>
              Debug
            </Link>
          </li>
        {/if}
      </ul>
    </nav>
  </div>
  <div slot="sidebar">
    <h1>{notNull(boundary).properties.name}</h1>

    {#if numDisconnectedCells > 0}
      <mark>
        Some parts of the neighbourhood aren't reachable by drivers, shown in
        red
      </mark>
    {/if}

    <h2>Editing tools</h2>
    <div class="tool-palette">
      <button
        on:click={() => (action = { kind: "filter" })}
        disabled={action.kind == "filter"}
        class:active={action.kind == "filter"}
        class:outline={action.kind != "filter"}
        data-tooltip="Add a modal filter (hotkey 1)"
      >
        <img
          src={`${import.meta.env.BASE_URL}/filters/${$filterType}_icon.gif`}
          alt="Add a modal filter"
        />
      </button>
      <button
        on:click={() => (action = { kind: "freehand-filters" })}
        disabled={action.kind == "freehand-filters"}
        class:active={action.kind == "freehand-filters"}
        class:outline={action.kind != "freehand-filters"}
        data-tooltip="Add many modal filters along a line (hotkey 2)"
      >
        <img
          src={`${import.meta.env.BASE_URL}/filters/select_freehand.png`}
          alt="Add many modal filters along a line"
        />
      </button>
      <button
        on:click={() => (action = { kind: "oneway" })}
        disabled={action.kind == "oneway"}
        class:active={action.kind == "oneway"}
        class:outline={action.kind != "oneway"}
        data-tooltip="Toggle one-way (hotkey 3)"
      >
        <!-- 
         cheat the default padding just a bit with negative placement, 
         these small circles crowd each other more than they crowd their container
         -->
        <div style="height: 100%; width: 100%; position: relative;">
          <img
            style="position: absolute; width: 60%; height: 60%; top: -1px; left: -1px;"
            src={onewayArrowUrl}
            alt="Reverse directions"
          />
          <img
            style="position: absolute; width: 60%; height: 60%; bottom: -1px; right: -1px; transform: rotate(180deg);"
            src={onewayArrowUrl}
            alt="Reverse directions"
          />
        </div>
      </button>
      <button
        on:click={() => (action = startTurnRestrictionAction())}
        disabled={action.kind == "turn_restriction"}
        class:active={action.kind == "turn_restriction"}
        class:outline={action.kind != "turn_restriction"}
        data-tooltip="Restrict turns (hotkey 4)"
      >
        <img
          src={`${import.meta.env.BASE_URL}/filters/no_right_turn.svg`}
          alt="Restrict turns"
        />
      </button>
      <button
        on:click={() => (action = { kind: "main-roads" })}
        disabled={action.kind == "main-roads"}
        class:active={action.kind == "main-roads"}
        class:outline={action.kind != "main-roads"}
        data-tooltip="Specify main roads (hotkey 5)"
      >
        <img src={onewayArrowUrl} alt="Change main/minor roads" />
      </button>
    </div>

    <button class="outline" on:click={() => (settingFilterType = true)}>
      Change modal filter type
    </button>
    <ChangeFilterModal bind:show={settingFilterType} />

    {#if action.kind == "main-roads"}
      <p>TODO explain what main roads mean, why changing it might make sense</p>
    {/if}

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

    <h2>Map style</h2>
    <label>
      <input type="checkbox" bind:checked={$animateShortcuts} />
      Animate shortcuts
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
        <SequentialLegend
          colorScale={speedColorScale}
          labels={{ limits: speedLimits }}
        />
      {/if}
    </div>

    <h2>Neighbourhood stats</h2>
    <NeighbourhoodBoundarySummary neighbourhoodBoundary={notNull(boundary)} />
  </div>

  <div slot="map">
    <MapEvents on:click={onMapClick} />

    <RenderNeighbourhood input={gj}>
      <HighlightBoundaryLayer />
      <CellLayer />
      <OneWayLayer />

      <NeighbourhoodRoadLayer
        interactive={action.kind == "filter" ||
          action.kind == "oneway" ||
          action.kind == "main-roads" ||
          (action.kind == "turn_restriction" && action.from_road_id == null)}
        {onClickLine}
      >
        <div slot="line-popup">
          <Popup openOn="hover" let:props>
            {#if props.kind == "interior_road"}
              <p>
                {props.shortcuts} shortcuts through {props.name ??
                  "unnamed road"}
                ({Math.round(props.speed_mph)} mph)
              </p>
            {:else if props.kind == "main_road"}
              <p>
                Main road: {props.name ?? "unnamed road"}
                ({Math.round(props.speed_mph)} mph)
              </p>
            {/if}
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
            {:else if action.kind == "main-roads"}
              <p>Click to toggle this a main / minor road</p>
            {:else if action.kind == "turn_restriction"}
              <p>Click to create a turn restriction from here</p>
            {/if}
          </Popup>
        </div>
      </NeighbourhoodRoadLayer>
      <EditableIntersectionLayer
        interactive={action.kind == "filter"}
        neighbourhood={gj}
        {onClickIntersection}
      />
    </RenderNeighbourhood>

    {#if $animateShortcuts}
      <AnimatePaths paths={allShortcuts} />
    {/if}

    <ModalFilterLayer
      onClickModalFilter={deleteModalFilter}
      onClickTurnRestriction={deleteTurnRestriction}
    >
      <div slot="modal-filter">
        <Popup openOn="hover">Click to delete</Popup>
      </div>
      <div slot="turn-restriction">
        <Popup openOn="hover">Click to delete</Popup>
      </div>
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
            <div>
              <img
                src={`${import.meta.env.BASE_URL}/filters/no_${props.kind}_turn.png`}
                width="20"
                alt="Add turn restriction"
              />

              Create a turn restriction from {action.from_road_name} to {props.name ||
                "unnamed road"}
            </div>
          </Popup>
        </LineLayer>
      </GeoJSON>
    {/if}
  </div>
</SplitComponent>

<style>
  .tool-palette {
    height: 60px;
    display: flex;
    justify-content: left;
    gap: 8px;
  }
  .tool-palette button {
    padding: 8px;
    margin: 0;
    height: 100%;
    aspect-ratio: 1;
  }
  .tool-palette button img {
    aspect-ratio: 1;
    width: 100%;
    object-fit: contain;
  }
  .tool-palette button.active:disabled {
    /* slightly increased border */
    border: 2px solid black;
    /* Slightly decreased padding to account for the slightly increased border */
    padding: 7px;

    /* picocss default color is very dark */
    background: rgb(124, 190, 146);
    /* picocss disabled override */
    opacity: 1;
  }
</style>
