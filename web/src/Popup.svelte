<script lang="ts" generics="PROPERTIES = Record<string, never>">
  import { Popup } from "svelte-maplibre";
  import type { Feature } from "geojson";
  import type { Snippet } from "svelte";

  // This wraps the svelte-maplibre Popup, passing the one feature's properties, which must be declared in a type-safe way.

  let props = $props<{
    children: Snippet<[{props: PROPERTIES}]>;
    openOn: 'hover' | 'click';
  }>();

  // TODO Why are there some blank popups sometimes? Should we use canOpen and check?
</script>

<Popup openOn={props.openOn}>
  {#snippet children({ data } : { data: Feature<any, PROPERTIES> | undefined })}
    {#if data}
      {@render props.children({ props: data.properties })}
    {/if}
  {/snippet}
</Popup>
