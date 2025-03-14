import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { projectStorage } from "./loader";

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
      setItem: vi.fn((key, value) => {
        storage[key] = value;
      }),
      removeItem: vi.fn((key) => {
        delete storage[key];
      }),
      clear: vi.fn(),
      key: vi.fn((index) => Object.keys(storage)[index] || null),
      get length() {
        return Object.keys(storage).length;
      },
    },
    writable: true,
  });

  return { storage, localStorage: window.localStorage };
}

beforeEach(() => {
  vi.resetAllMocks();
});

afterEach(() => {
  vi.resetAllMocks();
});

describe("Project listing and saving functionality", () => {
  describe("listProjects", () => {
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

      const result = projectStorage.listProjects("cnt");

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

      const result = projectStorage.listProjects("cnt");
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

      const studyAreas = projectStorage.listProjects("global");
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
      expect(projectStorage.listProjects("cnt")).toHaveLength(0);
      expect(projectStorage.listProjects("global")).toHaveLength(0);
    });
  });

  describe("createEmptyProject", () => {
    it("should create an empty project with study area name", () => {
      const { storage } = mockLocalStorage({});

      projectStorage.createEmptyProject("test-project", "TestArea");

      expect(storage["test-project"]).toBe(
        JSON.stringify({
          type: "FeatureCollection",
          features: [],
          study_area_name: "TestArea",
        }),
      );
    });

    it("should throw an error when key is empty", () => {
      mockLocalStorage({});
      expect(() => projectStorage.createEmptyProject("", "TestArea")).toThrow(
        "Cannot create project: no key specified",
      );
    });
  });

  describe("saveProject", () => {
    it("should save a string project to localStorage", () => {
      const { storage } = mockLocalStorage({});
      const projectData = JSON.stringify({
        type: "FeatureCollection",
        features: [],
        study_area_name: "TestArea",
      });

      projectStorage.saveProject("test-project", projectData);

      expect(storage["test-project"]).toBe(projectData);
    });

    it("should throw an error when key is empty", () => {
      mockLocalStorage({});
      expect(() => projectStorage.saveProject("", "{}")).toThrow(
        "Cannot save project: no key specified",
      );
    });
  });

  describe("removeProject", () => {
    it("should remove a project from localStorage", () => {
      mockLocalStorage({
        "ltn_cnt/Foo/Bar": {
          type: "FeatureCollection",
          features: [],
        },
      });

      expect(projectStorage.listProjects("cnt")).toHaveLength(1);
      projectStorage.removeProject("ltn_cnt/Foo/Bar");
      expect(projectStorage.listProjects("cnt")).toHaveLength(0);
    });

    it("should throw error when key is empty", () => {
      mockLocalStorage({});
      expect(() => projectStorage.removeProject("")).toThrow(
        "Cannot remove project: no key specified",
      );
    });

    it("should not throw error when project doesn't exist", () => {
      mockLocalStorage({}); // Empty localStorage
      expect(() => projectStorage.removeProject("nonexistent")).not.toThrow();
    });
  });

  describe("renameProject", () => {
    it("should rename a project by saving under new key and removing old one", () => {
      const { storage } = mockLocalStorage({
        "old-key": {
          type: "FeatureCollection",
          features: [],
          study_area_name: "TestArea",
        },
      });

      const oldData = storage["old-key"];

      projectStorage.renameProject("old-key", "new-key");

      expect(storage["new-key"]).toBe(oldData);
      expect(storage["old-key"]).toBeUndefined();
    });

    it("should throw error when old key is empty", () => {
      mockLocalStorage({});
      expect(() => projectStorage.renameProject("", "new-key")).toThrow(
        "Cannot rename project: keys must be specified",
      );
    });

    it("should throw error when new key is empty", () => {
      mockLocalStorage({});
      expect(() => projectStorage.renameProject("old-key", "")).toThrow(
        "Cannot rename project: keys must be specified",
      );
    });

    it("should throw error when project doesn't exist", () => {
      mockLocalStorage({}); // Empty localStorage
      expect(() =>
        projectStorage.renameProject("nonexistent", "new-key"),
      ).toThrow("Project nonexistent not found");
    });
  });
});
