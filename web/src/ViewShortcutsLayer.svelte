<script lang="ts">
  import { LTN } from "backend";
  import { GeoJSON, LineLayer } from "svelte-maplibre";
  import { choseRoad, state } from "./stores";

  export let app: LTN;
</script>

{#if $state.state == "neutral"}
  <GeoJSON data={JSON.parse(app.render())}>
    <LineLayer
      paint={{
        "line-width": 5,
        "line-color": "black",
      }}
      on:click={(e) => choseRoad(app, e.detail.features[0].properties.id)}
      hoverCursor="pointer"
    />
  </GeoJSON>
{:else if $state.state == "chose-road"}
  {#if $state.shortcutIndex == null}
    <GeoJSON data={$state.gj}>
      <LineLayer
        paint={{
          "line-width": 5,
          "line-color": "red",
        }}
      />
    </GeoJSON>
  {:else}
    <GeoJSON data={$state.gj.features[$state.shortcutIndex]}>
      <LineLayer
        paint={{
          "line-width": 5,
          "line-color": "red",
        }}
      />
    </GeoJSON>
  {/if}
{/if}
