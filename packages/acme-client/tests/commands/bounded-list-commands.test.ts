import { beforeEach, describe, expect, it, vi } from "vitest";

const { getMock } = vi.hoisted(() => ({ getMock: vi.fn() }));

vi.mock("../../src/utils/client-factory.js", () => ({
  getHttpClient: () => ({ get: getMock }),
  getAdminHttpClient: () => ({ get: getMock }),
}));

import { listPasskeys } from "../../src/commands/auth/passkey-commands.js";
import { listUsages } from "../../src/commands/media-commands.js";
import { listProjects, listTasks } from "../../src/commands/user-commands.js";
import { listUserSessions } from "../../src/commands/admin/user-commands.js";

describe("bounded list commands", () => {
  beforeEach(() => {
    getMock.mockReset();
  });

  it("walks all project pages for the front dashboard command", async () => {
    getMock
      .mockResolvedValueOnce({
        data: [{ id: "project-1" }],
        total: 2,
        hasMore: true,
      })
      .mockResolvedValueOnce({
        data: [{ id: "project-2" }],
        total: 2,
        hasMore: false,
      });

    const projects = await listProjects(fetch, "token");

    expect(projects.map((project) => project.id)).toEqual(["project-1", "project-2"]);
    expect(getMock.mock.calls.map(([path]) => path)).toEqual([
      "/v1/projects?page=1&limit=100",
      "/v1/projects?page=2&limit=100",
    ]);
  });

  it("walks all task pages for the front project detail command", async () => {
    getMock
      .mockResolvedValueOnce({
        data: [{ id: "task-1" }],
        total: 2,
        hasMore: true,
      })
      .mockResolvedValueOnce({
        data: [{ id: "task-2" }],
        total: 2,
        hasMore: false,
      });

    const tasks = await listTasks("project id", fetch, "token");

    expect(tasks.map((task) => task.id)).toEqual(["task-1", "task-2"]);
    expect(getMock.mock.calls.map(([path]) => path)).toEqual([
      "/v1/projects/project id/tasks?page=1&limit=100",
      "/v1/projects/project id/tasks?page=2&limit=100",
    ]);
  });

  it("walks all passkey pages so credential management sees the full account", async () => {
    getMock
      .mockResolvedValueOnce({
        data: [{ id: "credential-1" }],
        total: 2,
        hasMore: true,
      })
      .mockResolvedValueOnce({
        data: [{ id: "credential-2" }],
        total: 2,
        hasMore: false,
      });

    const passkeys = await listPasskeys(fetch, "token");

    expect(passkeys.map((passkey) => passkey.id)).toEqual(["credential-1", "credential-2"]);
    expect(getMock.mock.calls.map(([path]) => path)).toEqual([
      "/v1/auth/passkeys?page=1&limit=100",
      "/v1/auth/passkeys?page=2&limit=100",
    ]);
  });

  it("walks all media usage pages consumed by the media detail view", async () => {
    getMock
      .mockResolvedValueOnce({
        data: [{ id: "usage-1" }],
        total: 2,
        hasMore: true,
      })
      .mockResolvedValueOnce({
        data: [{ id: "usage-2" }],
        total: 2,
        hasMore: false,
      });

    const usages = await listUsages("media id", fetch, "token");

    expect(usages.data.map((usage) => usage.id)).toEqual(["usage-1", "usage-2"]);
    expect(usages.total).toBe(2);
    expect(usages.hasMore).toBe(false);
    expect(getMock.mock.calls.map(([path]) => path)).toEqual([
      "/v1/admin/media/media%20id/usage?page=1&limit=100",
      "/v1/admin/media/media%20id/usage?page=2&limit=100",
    ]);
  });

  it("passes the selected session page from the admin sessions template", async () => {
    getMock.mockResolvedValueOnce({ data: [{ id: "session-2" }], total: 25, hasMore: true });

    const sessions = await listUserSessions("user id", fetch, "token", { page: 2, limit: 10 });

    expect(sessions.data).toEqual([{ id: "session-2" }]);
    expect(getMock).toHaveBeenCalledWith(
      "/v1/admin/users/user%20id/sessions?page=2&limit=10",
    );
  });
});
