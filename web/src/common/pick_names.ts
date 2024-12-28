import type { Feature } from "geojson";
import type { Backend } from "../wasm";

// Returns a name for a neighbourhood, or null if the user cancels. Guaranteed to not overwrite an existing name.
export function pickNeighbourhoodName(
  backend: Backend,
  promptMessage: string,
  defaultValue: string,
): string | null {
  let used: Set<String> = new Set(
    backend
      .toSavefile()
      .features.filter((f: Feature) => f.properties!.kind == "boundary")
      .map((f: Feature) => f.properties!.name),
  );

  while (true) {
    let name = window.prompt(promptMessage, defaultValue);
    if (!name) {
      return null;
    }
    if (used.has(name)) {
      window.alert(
        `There is already a neighbourhood called ${name}; please pick another name`,
      );
    } else {
      return name;
    }
  }
}
