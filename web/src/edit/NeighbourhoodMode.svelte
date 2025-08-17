<script lang="ts">
  import type { Feature, FeatureCollection, LineString } from "geojson";
  import {
    Eraser,
    Paintbrush,
    Pointer,
    Redo,
    Route,
    Trash2,
    Undo,
  } from "lucide-svelte";
  import type { LngLat, MapMouseEvent } from "maplibre-gl";
  import { onDestroy, onMount } from "svelte";
  import {
    GeoJSON,
    hoverStateFilter,
    LineLayer,
    MapEvents,
    Popup,
    type LayerClickInfo,
  } from "svelte-maplibre";
  import { SequentialLegend } from "svelte-utils";
  import { emptyGeojson } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import eraserCursorURL from "../../assets/cursors/eraser.svg?url";
  import paintbrushCursorURL from "../../assets/cursors/paintbrush.svg?url";
  import noLeftUrl from "../../assets/filters/no_left_turn.png?url";
  import noRightUrl from "../../assets/filters/no_right_turn.png?url";
  import noStraightUrl from "../../assets/filters/no_straight_turn.png?url";
  import noUTurnRtlUrl from "../../assets/filters/no_u_left_to_right_turn.png?url";
  import noUTurnLtrUrl from "../../assets/filters/no_u_right_to_left_turn.png?url";
  import onewayArrowUrl from "../../assets/one_way_left.svg?url";
  import mainRoadIconUrl from "../../assets/traffic_1.svg?url";
  import AnimatePaths from "../AnimatePaths.svelte";
  import {
    initTooltips,
    layerId,
    ModeLink,
    pageTitle,
    roadLineWidth,
  } from "../common";
  import { speedColorScale, speedLimits } from "../common/colors";
  import type { Waypoint } from "../common/draw_area/stores";
  import type { Intersection } from "../common/Intersection";
  import { ModalFilterType } from "../common/ModalFilterType";
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
    currentFilterType,
    devMode,
    drawBorderEntries,
    map,
    mode,
    mutationCounter,
    roadStyle,
    saveCurrentProject,
    showBeforeEdits,
    thickRoadsForShortcuts,
  } from "../stores";
  import type {
    AllShortcuts,
    NeighbourhoodBoundaryFeature,
    RenderNeighbourhoodOutput,
  } from "../wasm";
  import ChangeFilterModal from "./ChangeFilterModal.svelte";
  import FreehandLine from "./FreehandLine.svelte";
  import ShowBeforeEdits from "./ShowBeforeEdits.svelte";
  import SnapRouteSelector from "./SnapRouteSelector.svelte";

  // Caller is responsible for doing backend.setCurrentNeighbourhood

  type Action =
    | { kind: "filter"; freehand: boolean }
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
    | {
        kind: "main-roads";
        tool: "toggle" | "snap-route-main" | "snap-route-erase";
        // Only used for snap-route-main or snap-route-erase
        waypoints: Waypoint[];
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
  let action: Action = { kind: "filter", freehand: false };

  $: if (action.kind == "oneway" || action.kind == "main-roads") {
    $map!.doubleClickZoom.disable();
  } else {
    $map!.doubleClickZoom.enable();
  }

  $: {
    if (action.kind == "filter" && action.freehand) {
      $map!.getCanvas().style.cursor = `url(${paintbrushCursorURL}) 8 22, cell`;
    } else if (
      action.kind == "main-roads" &&
      action.tool == "snap-route-erase"
    ) {
      $map!.getCanvas().style.cursor = `url(${eraserCursorURL}) 8 22, cell`;
    } else {
      $map!.getCanvas().style.cursor = "";
    }
  }

  let settingFilterType = false;
  let undoLength = 0;
  let redoLength = 0;
  let boundary: NeighbourhoodBoundaryFeature | null;

  let gj: RenderNeighbourhoodOutput;
  $: rerender($mutationCounter);

  let allShortcuts = emptyGeojson() as AllShortcuts;
  let lastShortcutCalculation = 0;
  $: recalculateShortcuts($mutationCounter, $animateShortcuts);

  $: numDisconnectedCells = gj.features.filter(
    (f) =>
      f.properties.kind == "cell" && f.properties.cell_color == "disconnected",
  ).length;

  onMount(() => {
    initTooltips();
  });
  onDestroy(() => {
    $map!.doubleClickZoom.enable();
  });

  function rerender(_x: number) {
    gj = $backend!.renderNeighbourhood();
    // @ts-expect-error TS can't figure out that we're narrowing the case here
    boundary = gj.features.find((f) => f.properties.kind == "boundary")!;

    undoLength = gj.undo_length;
    redoLength = gj.redo_length;

    saveCurrentProject();
  }

  function recalculateShortcuts(_x: number, animate: boolean) {
    if ($mutationCounter == lastShortcutCalculation || !animate) {
      return;
    }
    allShortcuts = $backend!.getAllShortcuts();
    lastShortcutCalculation = $mutationCounter;
  }

  function onClickLine(f: Feature, pt: LngLat) {
    if (action.kind == "filter") {
      $backend!.addModalFilter(pt, $currentFilterType);
      $mutationCounter++;
    } else if (action.kind == "oneway") {
      $backend!.toggleTravelFlow(f.properties!.road);
      $mutationCounter++;
    } else if (action.kind == "main-roads" && action.tool == "toggle") {
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

  function onMapClick(e: MapMouseEvent) {
    if (action.kind != "turn_restriction") {
      return;
    }

    // If we click a blank area, reset some state. Not sure why, but clicking
    // layers doesn't always prevent a click on the map itself.
    if (
      $map!.queryRenderedFeatures(e.point, {
        layers: ["interior-roads", "turn-restriction-targets"],
      }).length > 0
    ) {
      return;
    }

    action = startTurnRestrictionAction();
  }

  function createTurnRestriction(e: LayerClickInfo) {
    if (action.kind == "turn_restriction" && action.from_road_id != null) {
      let to = e.features[0].properties!.road;
      $backend!.addTurnRestriction(action.from_road_id, to);
      $mutationCounter++;
    }
    action = startTurnRestrictionAction();
  }

  function deleteModalFilter(e: LayerClickInfo) {
    let f = e.features[0];
    $backend!.deleteModalFilter(f.properties!.road);
    $mutationCounter++;
  }

  function deleteTurnRestriction(e: LayerClickInfo) {
    let f = e.features[0];
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

  function currentlyDoingMultiStepInteraction(action: Action): boolean {
    if (action.kind == "filter" && action.freehand) {
      return true;
    }
    if (
      action.kind == "main-roads" &&
      action.tool != "toggle" &&
      action.waypoints.length > 0
    ) {
      return true;
    }
    if (action.kind == "turn_restriction" && action.from_road_id != null) {
      return true;
    }
    return false;
  }

  function onKeyDown(e: KeyboardEvent) {
    if ($showBeforeEdits) {
      return;
    }
    // Ignore keypresses if we're not focused on the map
    if ((e.target as HTMLElement).tagName == "INPUT") {
      return;
    }

    // In the middle of more complex interactions, don't allow any keypresses
    if (currentlyDoingMultiStepInteraction(action)) {
      return;
    }

    if (e.ctrlKey && e.key == "z") {
      undo();
    }
    if (e.ctrlKey && e.key == "y") {
      redo();
    }
    if (e.key == "1") {
      action = { kind: "filter", freehand: false };
    }
    if (e.key == "2") {
      action = { kind: "oneway" };
    }
    if (e.key == "3") {
      action = startTurnRestrictionAction();
    }
    if (e.key == "4") {
      action = { kind: "main-roads", tool: "toggle", waypoints: [] };
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

  function paintedModalFiltersLine(e: CustomEvent<Feature<LineString> | null>) {
    let f = e.detail;
    if (f) {
      $backend!.addManyModalFilters(f, $currentFilterType);
      $mutationCounter++;
    }
    action = { kind: "filter", freehand: false };
  }

  function eraseAllMainRoads() {
    $backend!.eraseAllMainRoads();
    $mutationCounter++;
  }

  function finishSnapping(intersections: number[]) {
    let makeMainRoad =
      action.kind == "main-roads" && action.tool == "snap-route-main";
    $backend!.setMainRoads(intersections, makeMainRoad);
    $mutationCounter++;
  }

  let shortcutDescriptionText =
    "Shortcuts are routes from one main road to another, which cut through the neighborhood's interior.";
  let cellsDescriptionText =
    "Cells are the colored area reachable without travelling along a main road.";

  let turnRestrictionUrls: Record<any, string> = {
    left: noLeftUrl,
    right: noRightUrl,
    straight: noStraightUrl,
    u_left_to_right: noUTurnLtrUrl,
    u_right_to_left: noUTurnRtlUrl,
  };
</script>

<svelte:window onkeydown={onKeyDown} />

<SplitComponent>
  {#snippet top()}
    <div style="display: flex; justify-content: space-between;">
      <nav aria-label="breadcrumb">
        <ul>
          <li>
            <ModeLink mode={{ mode: "title" }} />
          </li>
          <li>
            <ModeLink mode={{ mode: "pick-neighbourhood" }} />
          </li>
          <li>
            {pageTitle($mode.mode)}
          </li>
        </ul>
      </nav>
      <nav>
        <ul>
          <li>
            <ModeLink mode={{ mode: "view-shortcuts" }} />
          </li>
          <li>
            <ModeLink mode={{ mode: "route", prevMode: "neighbourhood" }} />
          </li>
          <li>
            <ModeLink
              mode={{ mode: "predict-impact", prevMode: "neighbourhood" }}
            />
          </li>
          <li>
            <ModeLink mode={{ mode: "impact-one-destination" }} />
          </li>
          <li>
            <ModeLink
              mode={{
                mode: "set-boundary",
                name: boundary!.properties.name,
                existing: boundary!,
              }}
            />
          </li>
          {#if $devMode}
            <li>
              <ModeLink mode={{ mode: "debug-neighbourhood" }}>Debug</ModeLink>
            </li>
          {/if}
        </ul>
      </nav>
    </div>
  {/snippet}

  {#snippet left()}
    <div
      style="display: flex; justify-content: space-between; align-items: center"
    >
      <h2>Editing tools</h2>
      <label>
        <input
          type="checkbox"
          role="switch"
          bind:checked={$showBeforeEdits}
          disabled={currentlyDoingMultiStepInteraction(action)}
        />
        Show before edits
      </label>
    </div>
    <div class:edits-disabled={$showBeforeEdits}>
      <div
        class="tool-palette"
        style="display: flex; justify-content: space-between; flex-wrap: wrap; gap: 6px;"
      >
        <div
          style="height: 50px; display: flex; justify-content: left; gap: 6px;"
        >
          <button
            onclick={() => (action = { kind: "filter", freehand: false })}
            class="icon-btn"
            class:active={action.kind == "filter"}
            data-tippy-content="Add a modal filter (hotkey 1)"
          >
            <img
              src={ModalFilterType.getFilter($currentFilterType)!.iconURL}
              alt="Add a modal filter"
            />
          </button>
          <button
            onclick={() => (action = { kind: "oneway" })}
            class="icon-btn"
            class:active={action.kind == "oneway"}
            data-tippy-content="Toggle one-way (hotkey 2)"
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
            onclick={() => (action = startTurnRestrictionAction())}
            class="icon-btn"
            class:active={action.kind == "turn_restriction"}
            data-tippy-content="Restrict turns (hotkey 3)"
          >
            <img src={noRightUrl} alt="Restrict turns" />
          </button>
          <button
            onclick={() =>
              (action = { kind: "main-roads", tool: "toggle", waypoints: [] })}
            class="icon-btn"
            class:active={action.kind == "main-roads"}
            data-tippy-content="Reclassify main roads (hotkey 4)"
          >
            <img src={mainRoadIconUrl} alt="Change main/minor roads" />
          </button>
        </div>
        <div
          style="height: 50px; display: flex; justify-content: right; gap: 6px;"
        >
          <button
            class="outline icon-btn"
            disabled={undoLength == 0}
            onclick={undo}
            data-tippy-content={undoLength == 0
              ? "Undo Ctrl+Z"
              : `Undo (${undoLength}) Ctrl+Z`}
          >
            <Undo />
          </button>
          <button
            class="outline icon-btn"
            disabled={redoLength == 0}
            onclick={redo}
            data-tippy-content={redoLength == 0
              ? "Redo Ctrl+Y"
              : `Redo (${redoLength}) Ctrl+Y`}
          >
            <Redo />
          </button>
        </div>
      </div>

      <div
        style="min-height: 200px; padding-bottom: 16px; border-bottom: solid var(--pico-muted-border-color) 1px;"
      >
        {#if action.kind == "filter"}
          <h3>Add modal filter</h3>
          <p>
            Modal filters restrict what kind of traffic can pass through a road
            segment. Place them strategically to deter shortcuts through your
            neighbourhood.
          </p>
          <ChangeFilterModal bind:show={settingFilterType} />
          <div
            style="display: flex; gap: 8px; align-items: leading; flex-direction: column; width: fit-content;"
          >
            <button class="outline" onclick={() => (settingFilterType = true)}>
              Change modal filter type
            </button>

            <button
              onclick={action.freehand
                ? () => (action = { kind: "filter", freehand: false })
                : () => (action = { kind: "filter", freehand: true })}
              class:active={action.freehand}
              class:outline={!action.freehand}
              data-tippy-content="Add many modal filters along a line"
            >
              <div style="display: flex; align-items: center; gap: 8px;">
                <Paintbrush />
                <span>Add along a line</span>
              </div>
            </button>
          </div>
        {:else if action.kind == "oneway"}
          <h3>Toggle one-way</h3>
          <p>
            Click on a road segment to toggle its direction. This will change
            the direction of traffic flow on that road.
          </p>
        {:else if action.kind == "turn_restriction"}
          <h3>Restrict turns</h3>
          <p>
            To restrict certain turns, first click on the source road, then the
            destination road. Traffic will no longer be able to turn from the
            source road to the destination road.
          </p>
        {:else if action.kind == "main-roads"}
          <h3>Reclassify main roads</h3>
          <p>
            <i>Main roads</i>, drawn in grey, were classified automatically
            using data from
            <a href="https://openstreetmap.org/about" target="_blank"
              >OpenStreetMap</a
            >, but you can reclassify a road segment by clicking on it.
          </p>

          <p>
            Main roads are typically better suited to support higher levels of
            traffic than neighbourhood roads.
          </p>
          <div
            class="classification-buttons"
            style="display: flex; flex-direction: column; gap: 8px; justify-content: left;"
          >
            <button
              onclick={() => {
                action = { kind: "main-roads", tool: "toggle", waypoints: [] };
              }}
              class:active={action.tool == "toggle"}
              class:outline={action.tool != "toggle"}
              data-tippy-content="Click a road to reclassify it"
            >
              <div style="display: flex; align-items: center; gap: 8px;">
                <Pointer />
                <span>Toggle segment</span>
              </div>
            </button>

            <button
              onclick={() => {
                action = {
                  kind: "main-roads",
                  tool: "snap-route-main",
                  waypoints: [],
                };
              }}
              class:active={action.tool == "snap-route-main"}
              class:outline={action.tool != "snap-route-main"}
              data-tippy-content="Reclassify multiple roads by drawing a route crossing them"
            >
              <div style="display: flex; align-items: center; gap: 8px;">
                <Route />
                <span>Mark as main along a route</span>
              </div>
            </button>

            <button
              onclick={() => {
                action = {
                  kind: "main-roads",
                  tool: "snap-route-erase",
                  waypoints: [],
                };
              }}
              class:active={action.tool == "snap-route-erase"}
              class:outline={action.tool != "snap-route-erase"}
              data-tippy-content="Reclassify multiple roads by drawing a route crossing them"
            >
              <div style="display: flex; align-items: center; gap: 8px;">
                <Eraser />
                <span>Erase main classification</span>
              </div>
            </button>

            <button class:outline={true} onclick={eraseAllMainRoads}>
              <div style="display: flex; align-items: center; gap: 8px;">
                <Trash2 />
                <span>Erase all main roads</span>
              </div>
            </button>
          </div>
        {/if}
      </div>

      {#if numDisconnectedCells > 0}
        <mark>
          Some parts of the neighbourhood aren't reachable by drivers, shown in
          red
        </mark>
      {/if}
    </div>

    <h2>Map style</h2>
    <label>
      <input type="checkbox" bind:checked={$animateShortcuts} />
      Animate shortcuts<span
        class="footnote-ref"
        data-tippy-content={shortcutDescriptionText}>1</span
      >
    </label>

    <label>
      <input type="checkbox" bind:checked={$drawBorderEntries} />
      Show entries into cells<span
        class="footnote-ref"
        data-tippy-content={cellsDescriptionText}>2</span
      >
    </label>

    <label>
      <input type="checkbox" bind:checked={$thickRoadsForShortcuts} />
      Road thickness depends on shortcuts<span
        class="footnote-ref"
        data-tippy-content={shortcutDescriptionText}>1</span
      >
    </label>

    <label
      style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;"
    >
      <span style="text-wrap: nowrap;">Draw roads:</span>
      <select
        style="margin: 0; padding: 8px; width: auto;"
        bind:value={$roadStyle}
      >
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
    <h2>Neighbourhood stats</h2>
    <NeighbourhoodBoundarySummary neighbourhoodBoundary={boundary!} />
  {/snippet}

  {#snippet main()}
    <MapEvents onclick={onMapClick} />

    <ShowBeforeEdits />

    <RenderNeighbourhood input={gj}>
      <HighlightBoundaryLayer />
      <CellLayer show={!$showBeforeEdits} />
      <OneWayLayer show={!$showBeforeEdits} />

      <NeighbourhoodRoadLayer
        show={!$showBeforeEdits}
        interactive={(action.kind == "filter" && !action.freehand) ||
          action.kind == "oneway" ||
          (action.kind == "main-roads" && action.tool == "toggle") ||
          (action.kind == "turn_restriction" && action.from_road_id == null)}
        {onClickLine}
      >
        {#snippet linePopup()}
          <Popup openOn="hover">
            {#snippet children({ data })}
              {@const props = data!.properties!}
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
                    src={ModalFilterType.getFilter($currentFilterType)!.iconURL}
                    width="20"
                    alt="Add modal filter"
                  />
                  Click to add modal filter
                </div>
              {:else if action.kind == "oneway"}
                <p>Click to change direction</p>
              {:else if action.kind == "main-roads" && action.tool == "toggle"}
                <p>Click to designate a main road or not</p>
              {:else if action.kind == "turn_restriction"}
                <p>Click to create a turn restriction from here</p>
              {/if}
            {/snippet}
          </Popup>
        {/snippet}
      </NeighbourhoodRoadLayer>
      <EditableIntersectionLayer
        show={!$showBeforeEdits}
        interactive={action.kind == "filter"}
        neighbourhood={gj}
        {onClickIntersection}
      />
    </RenderNeighbourhood>

    {#if $animateShortcuts}
      <AnimatePaths paths={allShortcuts} />
    {/if}

    <ModalFilterLayer
      show={!$showBeforeEdits}
      onClickModalFilter={deleteModalFilter}
      onClickTurnRestriction={deleteTurnRestriction}
      interactive={action.kind == "filter"}
    >
      {#snippet modalFilterPopup()}
        <Popup openOn="hover">Click to delete filter</Popup>
      {/snippet}
      {#snippet turnRestrictionPopup()}
        <Popup openOn="hover">Click to delete turn restriction</Popup>
      {/snippet}
    </ModalFilterLayer>

    {#if action.kind == "filter" && action.freehand}
      <FreehandLine map={$map!} on:done={paintedModalFiltersLine} />
    {:else if action.kind == "main-roads" && action.tool != "toggle"}
      <SnapRouteSelector
        map={$map!}
        finish={finishSnapping}
        cancel={() =>
          (action = { kind: "main-roads", tool: "toggle", waypoints: [] })}
        bind:waypoints={action.waypoints}
      />
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
          onclick={createTurnRestriction}
        >
          <Popup openOn="hover">
            {#snippet children({ data })}
              {@const props = data!.properties!}
              <div>
                <img
                  src={turnRestrictionUrls[props.kind]}
                  width="20"
                  alt="Add turn restriction"
                />

                Create a turn restriction from {action.kind ==
                "turn_restriction"
                  ? action.from_road_name
                  : "???"} to {props.name || "unnamed road"}
              </div>
            {/snippet}
          </Popup>
        </LineLayer>
      </GeoJSON>
    {/if}
  {/snippet}
</SplitComponent>

<style>
  .classification-buttons {
    width: fit-content;
  }

  .footnote-ref {
    font-size: 70%;
    color: var(--pico-secondary);
    text-decoration: underline;
    cursor: help;
    position: relative;
    top: -12px;
    left: 4px;
  }

  .edits-disabled {
    opacity: 0.5;
    background-color: grey;
    pointer-events: none;
  }
</style>
