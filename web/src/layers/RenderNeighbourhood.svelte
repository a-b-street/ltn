<script lang="ts">
  import { setContext, type Snippet } from "svelte";
  import { GeoJSON } from "svelte-maplibre";
  import { backend } from "../stores";
  import type { RenderNeighbourhoodOutput } from "../wasm";

  // This component should act as the parent for most other layers, who
  // will get the raw GJ data by svelte context if needed. If input isn't
  // specified, the backend will be called.

  interface Props {
    input?: RenderNeighbourhoodOutput | null;
    children: Snippet;
  }

  let { input = null, children }: Props = $props();

  // TODO Might be more clear for edit/NeighbourhoodMode to just setContext itself
  let data = $derived(input || $backend!.renderNeighbourhood());
  // Do this before rendering children that do getContext
  $effect.pre(() => {
    setContext("neighbourhoodGj", data);
  });
</script>

<GeoJSON {data} generateId>
  {@render children()}
</GeoJSON>
