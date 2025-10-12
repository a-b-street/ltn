<script lang="ts">
  import type { Map } from "maplibre-gl";
  import { map as mapStore } from "../stores";

  async function start() {
    let map = $mapStore!;

    let canvas = map.getCanvas();
    let width = canvas.width;
    let height = canvas.height;

    let tiles = [];
    // Just try a manual 2x2
    tiles.push(await createImageBitmap(await mapToBlob(map)));
    panByPixels(map, width, 0);
    tiles.push(await createImageBitmap(await mapToBlob(map)));
    panByPixels(map, -width, height);
    tiles.push(await createImageBitmap(await mapToBlob(map)));
    panByPixels(map, width, 0);
    tiles.push(await createImageBitmap(await mapToBlob(map)));

    console.log("got em, now stitch");
    downloadScreenshot(await stitchBlobs(tiles, 2), "stitched.png");
  }

  function stitchBlobs(bitmaps: ImageBitmap[], columns: number): Promise<Blob> {
    return new Promise((resolve, reject) => {
      // Assume all tiles have same size
      let tileWidth = bitmaps[0].width;
      let tileHeight = bitmaps[0].height;
      let rows = Math.ceil(bitmaps.length / columns);

      let canvas = document.createElement("canvas");
      canvas.width = columns * tileWidth;
      canvas.height = rows * tileHeight;
      let ctx = canvas.getContext("2d");
      if (!ctx) {
        reject("no context");
        return;
      }

      bitmaps.forEach((bmp, i) => {
        let col = i % columns;
        let row = Math.floor(i / columns);
        ctx.drawImage(bmp, col * tileWidth, row * tileHeight);
      });

      canvas.toBlob((blob) => {
        if (blob) {
          resolve(blob);
        } else {
          reject("no blob");
        }
      });
    });
  }

  function mapToBlob(map: Map): Promise<Blob> {
    return new Promise((resolve, reject) => {
      map.redraw();
      map.once("idle", () => {
        map.getCanvas().toBlob((blob) => {
          if (blob) {
            resolve(blob);
          } else {
            reject("no blob");
          }
        });
      });
    });
  }

  function downloadScreenshot(blob: Blob, filename: string) {
    let a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = filename;
    a.click();
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
