<script lang="ts">
  import type { Map } from "maplibre-gl";
  import { map as mapStore } from "../stores";

  async function start() {
    let map = $mapStore!;

    let canvas = map.getCanvas();
    let width = canvas.width;
    let height = canvas.height;

    // Just try a manual 2x2
    await takeScreenshot(map, "row1col1.png");
    panByPixels(map, width, 0);
    await takeScreenshot(map, "row1col2.png");
    panByPixels(map, -width, height);
    await takeScreenshot(map, "row2col1.png");
    panByPixels(map, width, 0);
    await takeScreenshot(map, "row2col2.png");
  }

  function takeScreenshot(map: Map, filename: string): Promise<void> {
    return new Promise((resolve, reject) => {
      map.redraw();
      map.once("idle", () => {
        map.getCanvas().toBlob((blob) => {
          const a = document.createElement("a");
          a.href = URL.createObjectURL(blob);
          a.download = filename;
          a.click();

          resolve();
        });
      });
    });
  }

  function panByPixels(map: Map, dx: number, dy: number) {
    let centerScreen = map.project(map.getCenter());
    map.setCenter(
      map.unproject({
        x: centerScreen.x + dx,
        y: centerScreen.y + dy,
      }),
    );
  }
</script>

<button class="outline" onclick={start}>Screenshot</button>
