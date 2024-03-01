<script lang="ts">
  import { maptilerBasemap } from "../stores";

  // TODO Z-ordering won't work when we change, because layerId() doesn't get recalculated
</script>

<label>
  Basemap:
  <select bind:value={$maptilerBasemap}>
    <option value="dataviz">MapTiler Dataviz</option>
    <option value="streets">MapTiler Streets</option>
    <option value="hybrid">MapTiler Satellite</option>
    <option value="uk-openzoomstack-light">OS Open Zoomstack</option>
  </select>
</label>
