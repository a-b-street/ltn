import { LTN } from "backend";
import { writable, type Writable } from "svelte/store";

export type State =
  | {
      state: "neutral";
    }
  | {
      state: "chose-road";
      road: number;
      gj: FeatureCollection;
      shortcutIndex: number | null;
    };

export let state: Writable<State> = writable({ state: "neutral" });

export function choseRoad(app: LTN, road: number) {
  let gj = JSON.parse(app.getShortcutsCrossingRoad(road));
  if (gj.features.length == 0) {
    window.alert("No shortcuts here");
    return;
  }

  state.set({
    state: "chose-road",
    road,
    gj,
    shortcutIndex: null,
  });
}
