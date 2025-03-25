import type { FeatureCollection } from "geojson";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { NeighbourhoodDefinitionFeature } from "../wasm";
import { Database, type ProjectFeatureCollection } from "./ProjectStorage";

// Helper function to mock localStorage with predefined data
function mockProjectStorage(mockData = {}) {
  let { storage, localStorage } = mockLocalStorage(mockData);
  let projectStorage = new Database().projectStorage("cnt");
  return { projectStorage, storage, localStorage };
}

function mockLocalStorage(mockData = {}) {
  let storage: Record<string, string> = {};

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
      clear: vi.fn(() => (storage = {})),
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

describe("Database", () => {
  describe("ensureMigrated", () => {
    it("should migrate projects from schema v0 to v1", () => {
      const mockData = {
        // Old schema format for global projects
        ltn_TestProject: {
          type: "FeatureCollection",
          features: [],
          study_area_name: "London",
        },
        // Edge case: project name starting with 'ltn_' - we were seeing this with what look to be file imports.
        ltn_ltn_ProjectWithPrefix: {
          type: "FeatureCollection",
          features: [],
          study_area_name: "Manchester",
        },
        // Old schema format for CNT projects (Scottish cities)
        "ltn_cnt/Edinburgh/TestCntProject": {
          type: "FeatureCollection",
          features: [],
        },
      };

      const { storage } = mockLocalStorage(mockData);

      // Reset projectStorage to trigger migration
      const database = new Database();
      database.ensureMigrated();

      // Should have schema version stored
      expect(storage["ltn/_meta/schema-version"]).toBe("2");

      // Old format keys should be removed
      expect(storage["ltn_TestProject"]).toBeUndefined();
      expect(storage["ltn_ltn_ProjectWithPrefix"]).toBeUndefined();
      expect(storage["ltn_cnt/Edinburgh/TestCntProject"]).toBeUndefined();

      // Check global projects were migrated correctly
      const globalStorage = database.projectStorage("global");
      const globalProjects = globalStorage.studyAreaProjects();

      // Find London and Manchester study areas
      const londonArea = globalProjects.find(([name]) => name === "London");
      const manchesterArea = globalProjects.find(
        ([name]) => name === "Manchester",
      );

      expect(londonArea).toBeDefined();
      expect(manchesterArea).toBeDefined();

      // Check project names were preserved correctly
      const londonProject = londonArea![1][0];
      expect(londonProject.projectName).toBe("TestProject");

      const manchesterProject = manchesterArea![1][0];
      expect(manchesterProject.projectName).toBe("ProjectWithPrefix");

      // Check CNT projects were migrated correctly
      const cntStorage = database.projectStorage("cnt");
      const cntProjects = cntStorage.studyAreaProjects();

      const cntEdinburghArea = cntProjects.find(
        ([name]) => name === "Edinburgh",
      );
      expect(cntEdinburghArea).toBeDefined();

      const cntEdinburghProject = cntEdinburghArea![1][0];
      expect(cntEdinburghProject.projectName).toBe("TestCntProject");
    });

    it("should not migrate if schema is already up to date", () => {
      mockLocalStorage();

      let database = new Database();
      database.ensureMigrated();

      const getItemSpy = vi.spyOn(localStorage, "getItem");
      const setItemSpy = vi.spyOn(localStorage, "setItem");

      database = new Database();
      database.ensureMigrated();

      // Should check schema version
      expect(getItemSpy).toHaveBeenCalledWith("ltn/_meta/schema-version");
      // Should not set schema version again
      expect(setItemSpy).not.toHaveBeenCalledWith(
        "ltn/_meta/schema-version",
        expect.anything(),
      );
    });

    it("should set schema version when localStorage is empty", () => {
      const { localStorage } = mockLocalStorage();
      localStorage.clear();
      const setItemSpy = vi.spyOn(localStorage, "setItem");

      const database = new Database();
      database.ensureMigrated();

      expect(setItemSpy).toHaveBeenCalledWith("ltn/_meta/schema-version", "2");
    });
  });
});

describe("ProjectStorage", () => {
  describe("createEmptyProject", () => {
    it("should save project data if it's a known project", () => {
      let { projectStorage, storage } = mockProjectStorage();
      let id = projectStorage.createEmptyProject("Project Name", "TestArea");
      let key = projectStorage.projectKey(id);
      let originalProject: ProjectFeatureCollection = JSON.parse(storage[key]);
      expect(originalProject).toBeDefined();
      expect(originalProject.features).toHaveLength(0);
      originalProject.features.push("FakeFeature" as any);
      projectStorage.saveProject(id, originalProject);

      let reloadedProject: FeatureCollection = JSON.parse(storage[key]);
      expect(reloadedProject.features).toHaveLength(1);
    });
  });

  describe("studyAreas", () => {
    it("should list projects by study area", () => {
      let { projectStorage } = mockProjectStorage();
      projectStorage.createEmptyProject("Project 1", "Edinburgh");
      projectStorage.createEmptyProject("Project 2", "Glasgow");
      projectStorage.createEmptyProject("Project 3", "Edinburgh");

      const result = projectStorage.studyAreaProjects();
      console.log(result);
      expect(result).toHaveLength(2); // Two study areas

      let edinburgh = result[0];
      expect(edinburgh[0]).toBe("Edinburgh");
      let edinburghProjects = edinburgh[1];
      expect(edinburghProjects).toHaveLength(2);
      expect(edinburghProjects[0].projectName).toBe("Project 1");
      expect(edinburghProjects[1].projectName).toBe("Project 3");

      let glasgow = result[1];
      expect(glasgow[0]).toBe("Glasgow");
      let glasgowProjects = glasgow[1];
      expect(glasgowProjects).toHaveLength(1);
      expect(glasgowProjects[0].projectName).toBe("Project 2");
    });

    it("should return empty array when storage is empty", () => {
      let { projectStorage } = mockProjectStorage();
      const result = projectStorage.studyAreaProjects();
      expect(result).toHaveLength(0);
    });
  });

  describe("removeProject", () => {
    it("should remove project", () => {
      let { projectStorage, storage } = mockProjectStorage();
      let id = projectStorage.createEmptyProject("Project Name", "TestArea");
      let key = projectStorage.projectKey(id);
      expect(storage[key]).toBeDefined();

      projectStorage.removeProject(id);
      expect(storage[key]).not.toBeDefined();
    });

    it("should not throw error when project doesn't exist", () => {
      let { projectStorage } = mockProjectStorage();
      expect(() =>
        projectStorage.removeProject("ce-nest-pas-un-uuid"),
      ).not.toThrow();
    });
  });

  describe("renameProject", () => {
    it("should rename a project by updating the projectSummary", () => {
      let { projectStorage } = mockProjectStorage();
      let id = projectStorage.createEmptyProject("Original Name", "TestArea");
      expect(projectStorage.projectName(id)).toBe("Original Name");

      projectStorage.renameProject(id, "New Name");
      expect(projectStorage.projectName(id)).toBe("New Name");
    });

    it("should throw error when project doesn't exist", () => {
      let { projectStorage } = mockProjectStorage();
      expect(() =>
        projectStorage.renameProject("ce-nest-pas-un-uuid", "New Name"),
      ).toThrow("Cannot get project: no project found for ce-nest-pas-un-uuid");
    });
  });

  describe("projectNameAlreadyExists", () => {
    it("should return true if project name already exists", () => {
      let { projectStorage } = mockProjectStorage();
      projectStorage.createEmptyProject("Existing Project", "TestArea");
      expect(projectStorage.projectNameAlreadyExists("Existing Project")).toBe(
        true,
      );
    });

    it("should return false if project name does not exist", () => {
      let { projectStorage } = mockProjectStorage();
      expect(
        projectStorage.projectNameAlreadyExists("Nonexistent Project"),
      ).toBe(false);
    });
  });

  describe("nextAvailableProjectName", () => {
    it("should return the same name if it's available", () => {
      let { projectStorage } = mockProjectStorage();
      expect(projectStorage.nextAvailableProjectName("Test Project")).toBe(
        "Test Project",
      );
    });
    it("should return a unique name if the name is already taken", () => {
      let { projectStorage } = mockProjectStorage();
      projectStorage.createEmptyProject("Test Project", "TestArea");
      expect(projectStorage.nextAvailableProjectName("Test Project")).toBe(
        "Test Project (2)",
      );
    });
    it("should return a unique name if the name is already taken multiple times", () => {
      let { projectStorage } = mockProjectStorage();
      projectStorage.createEmptyProject("Test Project", "TestArea");
      projectStorage.createEmptyProject("Test Project (2)", "TestArea");
      expect(projectStorage.nextAvailableProjectName("Test Project")).toBe(
        "Test Project (3)",
      );
    });
  });

  describe("nextAvailableNeighbourhoodName", () => {
    it("should return un-suffixed name when there are no neighbourhoods", () => {
      let { projectStorage } = mockProjectStorage();
      let projectID = projectStorage.createEmptyProject(
        "Test Project",
        "TestArea",
      );
      expect(projectStorage.nextAvailableNeighbourhoodName(projectID)).toBe(
        "Test Project LTN",
      );
    });

    it("should add a unique suffix if the name is already taken", () => {
      let { projectStorage } = mockProjectStorage();
      let projectID = projectStorage.createEmptyProject(
        "Test Project",
        "TestArea",
      );
      let existingNeighbourhood: NeighbourhoodDefinitionFeature = {
        type: "Feature",
        geometry: { type: "Polygon", coordinates: [] },
        properties: {
          kind: "boundary",
          name: "Test Project LTN",
        },
      };
      let project = projectStorage.project(projectID);
      project.features.push(existingNeighbourhood);
      projectStorage.saveProject(projectID, project);
      expect(projectStorage.nextAvailableNeighbourhoodName(projectID)).toBe(
        "Test Project LTN #2",
      );
    });

    it("should return un-suffixed name as long as any existing projects don't have that name.", () => {
      let { projectStorage } = mockProjectStorage();

      let projectID = projectStorage.createEmptyProject(
        "Test Project",
        "TestArea",
      );
      let existingNeighbourhood: NeighbourhoodDefinitionFeature = {
        type: "Feature",
        geometry: { type: "Polygon", coordinates: [] },
        properties: {
          kind: "boundary",
          name: "Custom neighbourhood name",
        },
      };
      let project = projectStorage.project(projectID);
      project.features.push(existingNeighbourhood);
      projectStorage.saveProject(projectID, project);
      expect(projectStorage.nextAvailableNeighbourhoodName(projectID)).toBe(
        "Test Project LTN",
      );
    });
  });
});
