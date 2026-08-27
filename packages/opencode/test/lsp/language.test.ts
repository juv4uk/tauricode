import { describe, expect, it } from "bun:test"
import { LANGUAGE_EXTENSIONS } from "@/lsp/language"
import { toolFiletype } from "@/cli/cmd/run/tool"

describe("LANGUAGE_EXTENSIONS wsm mapping", () => {
  it("maps .wsm to wsm", () => {
    expect(LANGUAGE_EXTENSIONS[".wsm"]).toBe("wsm")
  })
  it("maps .my to wsm (alias)", () => {
    expect(LANGUAGE_EXTENSIONS[".my"]).toBe("wsm")
  })
  it("maps .lisp to wsm (alias)", () => {
    expect(LANGUAGE_EXTENSIONS[".lisp"]).toBe("wsm")
  })
})

describe("toolFiletype wsm recognition", () => {
  it("returns wsm for .wsm files", () => {
    expect(toolFiletype("foo.wsm")).toBe("wsm")
  })
  it("returns wsm for .my files", () => {
    expect(toolFiletype("foo.my")).toBe("wsm")
  })
  it("returns wsm for .lisp files", () => {
    expect(toolFiletype("foo.lisp")).toBe("wsm")
  })
  it("returns undefined for unknown extensions", () => {
    expect(toolFiletype("foo.xyz")).toBeUndefined()
  })
})
