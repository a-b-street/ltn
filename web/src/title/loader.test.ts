import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { getProjectList } from "./loader";

// Helper function to mock localStorage with predefined data
function mockLocalStorage(mockData = {}) {
  const storage: Record<string, string> = {};

  Object.entries(mockData).forEach(([key, value]) => {
    storage[key] = JSON.stringify(value);
  });

  Object.defineProperty(window, "localStorage", {
    // mock localStorage API's that we use
    value: {
      getItem: vi.fn((key) => storage[key] || null),
      setItem: vi.fn(),
      removeItem: vi.fn(),
      clear: vi.fn(),
      key: vi.fn((index) => Object.keys(storage)[index] || null),
      length: Object.keys(storage).length,
    },
    writable: true,
  });

  return window.localStorage;
}

beforeEach(() => {
  vi.resetAllMocks();
  mockLocalStorage(); // Initialize with empty storage
});

afterEach(() => {
  vi.resetAllMocks();
});

describe("Project listing functionality", () => {
  describe("getProjectList", () => {
    it("should group CNT projects by study area", () => {
      mockLocalStorage({
        "ltn_cnt/LAD_Edinburgh/project1": {
          type: "FeatureCollection",
          features: [],
          study_area_name: "Edinburgh",
        },
        "ltn_cnt/LAD_Glasgow/project2": {
          type: "FeatureCollection",
          features: [],
          study_area_name: "Glasgow",
        },
        "ltn_cnt/LAD_Edinburgh/project3": {
          type: "FeatureCollection",
          features: [],
          study_area_name: "Edinburgh",
        },
        ltn_other_key: { ignore: "me" },
      });

      const result = getProjectList("cnt");

      expect(result).toHaveLength(2); // Two study areas

      const edinburghGroup = result.find((g) => g[0] === "Edinburgh");
      expect(edinburghGroup).toBeDefined();
      expect(edinburghGroup?.[1]).toHaveLength(2);

      const glasgowGroup = result.find((g) => g[0] === "Glasgow");
      expect(glasgowGroup).toBeDefined();
      expect(glasgowGroup?.[1]).toHaveLength(1);
    });

    it("should filter out non-CNT projects when appFocus is cnt", () => {
      mockLocalStorage({
        "ltn_cnt/LAD_Edinburgh/project1": {
          type: "FeatureCollection",
          features: [],
          study_area_name: "Edinburgh",
        },
        ltn_project1: {
          type: "FeatureCollection",
          features: [],
          study_area_name: "Global",
        },
      });

      const result = getProjectList("cnt");
      expect(result).toHaveLength(1);
      expect(result[0][0]).toBe("Edinburgh");
    });

    it("should handle global projects correctly", () => {
      mockLocalStorage({
        ltn_project1: {
          type: "FeatureCollection",
          features: [],
          study_area_name: "Bristol",
        },
        ltn_project2: {
          type: "FeatureCollection",
          features: [],
          study_area_name: "Bristol",
        },
        ltn_project3: {
          type: "FeatureCollection",
          features: [],
          study_area_name: "Seattle",
        },
        "ltn_cnt/LAD_Edinburgh/project1": {
          type: "FeatureCollection",
          features: [],
        },
        ltn_project4: {
          type: "FeatureCollection",
          features: [],
        },
      });

      const studyAreas = getProjectList("global");
      expect(studyAreas).toHaveLength(3);

      expect(studyAreas[0]).toEqual([
        "Bristol",
        [
          { projectId: "ltn_project1", projectName: "project1" },
          { projectId: "ltn_project2", projectName: "project2" },
        ],
      ]);

      expect(studyAreas[1]).toEqual([
        "Seattle",
        [{ projectId: "ltn_project3", projectName: "project3" }],
      ]);

      // custom area
      expect(studyAreas[2]).toEqual([
        undefined,
        [{ projectId: "ltn_project4", projectName: "project4" }],
      ]);
    });

    it("should handle empty localStorage", () => {
      mockLocalStorage({}); // Empty localStorage
      expect(getProjectList("cnt")).toHaveLength(0);
      expect(getProjectList("global")).toHaveLength(0);
    });
  });
});
