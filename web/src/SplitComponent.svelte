<script lang="ts">
  import { mapContents, sidebarContents } from "./stores";

  // The kinda weird hack is that this must itself be nested underneath the
  // MapLibre bit, so it has context. The sidebar is the "remote" part.
  // TODO Maybe simplify accordingly
</script>

<div bind:this={$sidebarContents}>
  <slot name="sidebar" />
</div>
<div bind:this={$mapContents}>
  <slot name="map" />
</div>
