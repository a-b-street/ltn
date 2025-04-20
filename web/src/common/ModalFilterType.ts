import busGateIconUrl from "../../assets/filters/bus_gate_icon.gif?url";
import busGateImageUrl from "../../assets/filters/bus_gate.gif?url";
import noEntryIconUrl from "../../assets/filters/no_entry_icon.gif?url";
import noEntryImageUrl from "../../assets/filters/no_entry.gif?url";
import schoolStreetIconUrl from "../../assets/filters/school_street_icon.gif?url";
import schoolStreetImageUrl from "../../assets/filters/school_street.gif?url";
import walkCycleOnlyIconUrl from "../../assets/filters/walk_cycle_only_icon.gif?url";
import walkCycleOnlyImageUrl from "../../assets/filters/walk_cycle_only.gif?url";

export class ModalFilterType {
  filterType: string;
  label: string;
  description: string;
  imageUrl: string;
  iconUrl: string;

  constructor(
    filterType: string,
    label: string,
    description: string,
    imageUrl: string,
    iconUrl: string,
  ) {
    this.filterType = filterType;
    this.label = label;
    this.description = description;
    this.imageUrl = imageUrl;
    this.iconUrl = iconUrl;
  }

  get iconURL(): string {
    return this.iconUrl;
  }

  get largeImageURL(): string {
    return this.imageUrl;
  }

  static getFilter(filterType: string): ModalFilterType | undefined {
    switch (filterType) {
      case "walk_cycle_only":
        return ModalFilterType.walkCycleOnly;
      case "no_entry":
        return ModalFilterType.noEntry;
      case "bus_gate":
        return ModalFilterType.busGate;
      case "school_street":
        return ModalFilterType.schoolStreet;
      default:
        console.assert(`unknown filter type: "${filterType}"`);
        return undefined;
    }
  }

  static walkCycleOnly = new ModalFilterType(
    "walk_cycle_only",
    "Walking/cycling only",
    "A physical barrier that only allows people walking, cycling, and rolling to pass. Often planters or bollards. Larger vehicles cannot enter.",
    walkCycleOnlyImageUrl,
    walkCycleOnlyIconUrl,
  );
  static noEntry = new ModalFilterType(
    "no_entry",
    "No entry",
    "An alternative sign to indicate vehicles are not allowed to enter the street. Only people walking, cycling, and rolling may pass through.",
    noEntryImageUrl,
    noEntryIconUrl,
  );
  static busGate = new ModalFilterType(
    "bus_gate",
    "Bus gate",
    "A bus gate sign and traffic cameras are installed to allow buses, pedestrians, and cyclists to pass. There is no physical barrier.",
    busGateImageUrl,
    busGateIconUrl,
  );
  static schoolStreet = new ModalFilterType(
    "school_street",
    "School street",
    "A closure during school hours only. The barrier usually allows teachers and staff to access the school.",
    schoolStreetImageUrl,
    schoolStreetIconUrl,
  );

  static allTypes = [
    ModalFilterType.walkCycleOnly,
    ModalFilterType.noEntry,
    ModalFilterType.busGate,
    ModalFilterType.schoolStreet,
  ];
}
