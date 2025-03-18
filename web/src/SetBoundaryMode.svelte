<script lang="ts">
  import type { Feature, Polygon } from "geojson";
  import type { AreaProps } from "route-snapper-ts";
  import { notNull } from "svelte-utils";
  import { gjPosition, Link } from "./common";
  import AreaControls from "./common/draw_area/AreaControls.svelte";
  import { calculateArea, type Waypoint } from "./common/draw_area/stores";
  import {
    backend,
    map,
    mode,
    returnToChooseProject,
    saveCurrentProject,
  } from "./stores";

  export let name: string;
  export let existing: Feature<Polygon, AreaProps> | null;
  let waypoints: Waypoint[] = [];

  if (existing) {
    if (existing.properties.waypoints) {
      // Transform into the correct format
      waypoints = existing.properties.waypoints.map((waypt) => {
        return {
          point: [waypt.lon, waypt.lat],
          snapped: waypt.snapped,
        };
      });
    } else {
      // No stored waypoints -- this is either a boundary drawn with a very old
      // version of this tool, or an auto-generated boundary. Just
      // "backfill" by using the full geometry as freehand points.
      // Editing will be very painful in practice, but it won't break.
      // Note the second polygon ring is used, because the boundary is expressed as
      // "everywhere" minus a hole for the boundary, to achieve the fade-outside effect.
      waypoints = existing.geometry.coordinates[1].slice(1).map((point) => {
        return { point: gjPosition(point), snapped: false };
      });
    }
  }

  function finish() {
    if (waypoints.length >= 3) {
      try {
        let feature = calculateArea(waypoints);
        $backend!.setNeighbourhoodBoundary(name, feature);
        saveCurrentProject();
        $backend!.setCurrentNeighbourhood(name);
        $mode = {
          mode: "neighbourhood",
        };
      } catch (err) {
        window.alert(`Sorry, this boundary is invalid: ${err}`);
        cancel();
      }
    }
  }

  function cancel() {
    if (existing) {
      $mode = {
        mode: "neighbourhood",
      };
    } else {
      $mode = {
        mode: "pick-neighbourhood",
      };
    }
  }
</script>

<AreaControls map={notNull($map)} {finish} {cancel} bind:waypoints>
  <div slot="extra-top">
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
        {#if existing}
          <li>
            <Link on:click={() => ($mode = { mode: "neighbourhood" })}>
              Editing
            </Link>
          </li>
          <li>Changing neighbourhood boundary</li>
        {:else}
          <li>Creating new neighbourhood boundary</li>
        {/if}
      </ul>
    </nav>
  </div>

  <div slot="extra-sidebar">
    <h1>Draw your neighbourhood boundary for {name}</h1>
  </div>
</AreaControls>
