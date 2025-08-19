<script lang="ts">
  import { setContext, type Snippet } from "svelte";
  import { GeoJSON } from "svelte-maplibre";
  import { backend } from "../stores";
  import type { RenderNeighbourhoodOutput } from "../wasm";

  // This component should act as the parent for most other layers, who
  // will get the raw GJ data by svelte context if needed. If input isn't
  // specified, the backend will be called.
  export let input: RenderNeighbourhoodOutput | null = null;
  export let children: Snippet;

  // TODO Might be more clear for edit/NeighbourhoodMode to just setContext itself
  $: data = input || $backend!.renderNeighbourhood();
  $: setContext("neighbourhoodGj", data);
</script>

<GeoJSON {data} generateId>
  {@render children()}
</GeoJSON>
