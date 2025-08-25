<script lang="ts">
  import { run } from 'svelte/legacy';

  import { setContext, type Snippet } from "svelte";
  import { GeoJSON } from "svelte-maplibre";
  import { backend } from "../stores";
  import type { RenderNeighbourhoodOutput } from "../wasm";

  // This component should act as the parent for most other layers, who
  // will get the raw GJ data by svelte context if needed. If input isn't
  
  interface Props {
    // specified, the backend will be called.
    input?: RenderNeighbourhoodOutput | null;
    children: Snippet;
  }

  let { input = null, children }: Props = $props();

  // TODO Might be more clear for edit/NeighbourhoodMode to just setContext itself
  let data = $derived(input || $backend!.renderNeighbourhood());
  run(() => {
    setContext("neighbourhoodGj", data);
  });
</script>

<GeoJSON {data} generateId>
  {@render children()}
</GeoJSON>
