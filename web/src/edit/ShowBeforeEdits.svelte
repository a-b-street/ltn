<script lang="ts">
  import type { FeatureCollection } from "geojson";
  import { Popup } from "svelte-utils/map";
  import {
    CellLayer,
    ModalFilterLayer,
    NeighbourhoodRoadLayer,
    OneWayLayer,
    RenderNeighbourhood,
  } from "../layers";
  import { backend, showBeforeEdits } from "../stores";
  import type { RenderNeighbourhoodOutput } from "../wasm";

  let prefix = "before-edits-";

  let neighbourhoodGj: RenderNeighbourhoodOutput | null = null;
  let modalFilterGj: FeatureCollection | null = null;
  let turnRestrictionGj: FeatureCollection | null = null;

  $: if ($showBeforeEdits && neighbourhoodGj == null) {
    neighbourhoodGj = $backend!.renderNeighbourhoodBeforeEdits();
    modalFilterGj = $backend!.renderModalFiltersBeforeEdits();
    turnRestrictionGj = $backend!.renderTurnRestrictionsBeforeEdits();
  }
</script>

{#if neighbourhoodGj}
  <RenderNeighbourhood input={neighbourhoodGj}>
    <CellLayer show={$showBeforeEdits} {prefix} />
    <OneWayLayer show={$showBeforeEdits} {prefix} />

    <NeighbourhoodRoadLayer show={$showBeforeEdits} {prefix} interactive={true}>
      <div slot="line-popup">
        <Popup openOn="hover" let:props>
          {#if props.kind == "interior_road"}
            <p>
              {props.shortcuts} shortcuts through {props.name ?? "unnamed road"}
              ({Math.round(props.speed_mph)} mph)
            </p>
          {:else if props.kind == "main_road"}
            <p>
              Main road: {props.name ?? "unnamed road"}
              ({Math.round(props.speed_mph)} mph)
            </p>
          {/if}
        </Popup>
      </div>
    </NeighbourhoodRoadLayer>
  </RenderNeighbourhood>
{/if}

{#if modalFilterGj && turnRestrictionGj}
  <ModalFilterLayer
    {prefix}
    show={$showBeforeEdits}
    interactive={false}
    {modalFilterGj}
    {turnRestrictionGj}
  />
{/if}
