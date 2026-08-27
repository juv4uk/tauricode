import { describe, test, expect, afterAll } from "bun:test"
import path from "path"
import fs from "fs/promises"
import os from "os"
import * as LSPServer from "@/lsp/server"
import type { InstanceContext } from "@/project/instance-context"

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const tmpBase = path.join(os.tmpdir(), "opencode-wsm-root-test")

function makeCtx(directory: string, worktree = "/"): InstanceContext {
  return { directory, worktree, project: {} as any }
}

async function mkdirp(p: string) {
  await fs.mkdir(p, { recursive: true })
}

async function touch(p: string) {
  await mkdirp(path.dirname(p))
  await fs.writeFile(p, "", "utf-8")
}

afterAll(async () => {
  await fs.rm(tmpBase, { recursive: true, force: true }).catch(() => {})
})

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("WsmLS.root", () => {
  test("non-git project (worktree sentinel '/') never resolves root to '/'", async () => {
    // Regression test: instance-context.ts documents "/" as the sentinel
    // for "no real git worktree boundary" (see its own containsPath
    // comment). A root() that blindly returns ctx.worktree whenever it
    // differs from ctx.directory hands "/" to the spawned LSP server as
    // its scan root -- and my-lisp-lsp indexes every source file under
    // its root on initialize, so "/" means an unbounded filesystem walk
    // that never returns. Confirmed live under WSL: the real process
    // wedged scanning /mnt/c/Windows/WinSxS instead of ever responding.
    const root = path.join(tmpBase, "no-git")
    await mkdirp(root)
    const srcDir = path.join(root, "src")
    await touch(path.join(srcDir, "inside.wsm"))

    const file = path.join(srcDir, "inside.wsm")
    const result = await LSPServer.WsmLS.root(file, makeCtx(root, "/"))
    expect(result).not.toBe("/")
    expect(result).toBe(root)
  })

  test("repo.my marker takes precedence over both worktree and instance directory", async () => {
    // ctx.directory is the outer boundary here (matches getClients passing
    // ctx.directory as NearestRoot's stop); repo.my lives in a nested
    // subdirectory strictly between the file and that boundary, so it must
    // win over both ctx.worktree and ctx.directory itself.
    const outer = path.join(tmpBase, "worktree-repo")
    const repoDir = path.join(outer, "nested", "repo")
    await mkdirp(repoDir)
    await touch(path.join(repoDir, "repo.my"))
    const srcDir = path.join(repoDir, "src")
    await touch(path.join(srcDir, "inside.wsm"))

    const file = path.join(srcDir, "inside.wsm")
    const result = await LSPServer.WsmLS.root(file, makeCtx(outer, outer))
    expect(result).toBe(repoDir)
  })

  test("real git worktree (not '/') is used when no repo.my marker is present", async () => {
    const worktree = path.join(tmpBase, "real-worktree")
    const instanceDir = path.join(worktree, "sub")
    const srcDir = path.join(instanceDir, "src")
    await touch(path.join(srcDir, "inside.wsm"))

    const file = path.join(srcDir, "inside.wsm")
    const result = await LSPServer.WsmLS.root(file, makeCtx(instanceDir, worktree))
    expect(result).toBe(worktree)
  })
})
