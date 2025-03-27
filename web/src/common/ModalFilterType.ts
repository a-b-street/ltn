// TODO Use of import.meta.env.BASE_URL below is to workaround https://github.com/vitejs/vite/issues/10601
export class ModalFilterType {
  filterType: string;
  label: string;
  description: string;

  constructor(filterType: string, label: string, description: string) {
    this.filterType = filterType;
    this.label = label;
    this.description = description;
  }

  get iconURL(): string {
    return `${import.meta.env.BASE_URL}/filters/${this.filterType}_icon.gif`;
  }

  get largeImageURL(): string {
    return `${import.meta.env.BASE_URL}/filters/${this.filterType}.gif`;
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
  );
  static noEntry = new ModalFilterType(
    "no_entry",
    "No entry",
    "An alternative sign to indicate vehicles are not allowed to enter the street. Only people walking, cycling, and rolling may pass through.",
  );
  static busGate = new ModalFilterType(
    "bus_gate",
    "Bus gate",
    "A bus gate sign and traffic cameras are installed to allow buses, pedestrians, and cyclists to pass. There is no physical barrier.",
  );
  static schoolStreet = new ModalFilterType(
    "school_street",
    "School street",
    "A closure during school hours only. The barrier usually allows teachers and staff to access the school.",
  );

  static allTypes = [
    ModalFilterType.walkCycleOnly,
    ModalFilterType.noEntry,
    ModalFilterType.busGate,
    ModalFilterType.schoolStreet,
  ];
}
