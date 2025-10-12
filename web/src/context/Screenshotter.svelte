<script lang="ts">
  import { map } from "../stores";

  function start() {
    $map.redraw();

    $map.once("idle", () => {
      $map.getCanvas().toBlob((blob) => {
        const a = document.createElement("a");
        a.href = URL.createObjectURL(blob);
        a.download = "screenshot.png";
        a.click();
      });
    });
  }
</script>

<button class="outline" onclick={start}>Screenshot</button>
