import { describe, expect, it } from "vitest";

import { readPush, refusePush } from "../src/push";

const OWN = "refs/heads/feedback/abc123def456";
const A = "ac6a8bf1e27bc5ebbe1cc1e60b16ebe52002bcc8";
const Z = "0".repeat(40);

// Request bodies recorded from git 2.43 pushing to `git http-backend` (first 400 bytes: ref list and pack start).
const RECORDED = {
  own: "MDBjMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAgYWM2YThiZjFlMjdiYzVlYmJlMWNjMWU2MGIxNmViZTUyMDAyYmNjOCByZWZzL2hlYWRzL2ZlZWRiYWNrL2FiYzEyM2RlZjQ1NgAgcmVwb3J0LXN0YXR1cy12MiBzaWRlLWJhbmQtNjRrIHF1aWV0IG9iamVjdC1mb3JtYXQ9c2hhMSBhZ2VudD1naXQvMi40My4wMDAwMFBBQ0sAAAACAAAAA5QgeJyVkM2OmzAURvd+Cu+jVtgYDNK0ahJDwkxgmoKZSXbGNiQBAuVn1OHpC1Opq256pCtdHem7uvqGTmsokMpzSYVNpJFhZboWcUiWEwtrV+gMKWEK5OQYtKLT9wFmhpPbttYIKZoTJZXrUqGwwNS2iGtmrrBsqagNxDhcmg5uKzEqDR/uTafb6v2buA+Xrmmv8rNs6q8QUdd0HGRhDFfGDJhtfR0G/f/Joi36awE/LWy8XRDBON7DONhF64T/8A==",
  main: "MDBhZjAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAgYjA4ZjY2ZWUxMWQ3ZjRkY2Q5OTdhZDJhMjc2NTQ5M2I5YTU2Y2Q3NiByZWZzL2hlYWRzL21haW4AIHJlcG9ydC1zdGF0dXMtdjIgc2lkZS1iYW5kLTY0ayBxdWlldCBvYmplY3QtZm9ybWF0PXNoYTEgYWdlbnQ9Z2l0LzIuNDMuMDAwMDBQQUNLAAAAAgAAAAKUHXiclZBLb4JAFIX38ytmb9owI48haZuKIKKAUsCm7mZgQAsK8rDCry/YpKtuepKbnHzJubn3NBXnUGQES3EkizhiMmcqFWSBSyJLSCyrmJCEcS6qggho2xyKCs5z2sYcPp2Lipd590rPzaEqymP0GBWnF4gUdUoIkjCGE2EQGOjp2DT8/8m0TOtjCh9GaYZpudD3l9C3THcWhG/GnQMYItcKhXw5G+UNozujs6OpW9jvu8z5DJETZCNLK7uvk4XlL0TWEW+yWPdY+SizNQr9Wg==",
  deleteOther:
    "MDBiMGFjNmE4YmYxZTI3YmM1ZWJiZTFjYzFlNjBiMTZlYmU1MjAwMmJjYzggMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMCByZWZzL2hlYWRzL290aGVyACByZXBvcnQtc3RhdHVzLXYyIHNpZGUtYmFuZC02NGsgcXVpZXQgb2JqZWN0LWZvcm1hdD1zaGExIGFnZW50PWdpdC8yLjQzLjAwMDAw",
};

function recorded(name: keyof typeof RECORDED): Uint8Array {
  return Uint8Array.from(atob(RECORDED[name]), (c) => c.charCodeAt(0));
}

function pkt(...lines: string[]): Uint8Array {
  const body = lines
    .map((line) => (line === "" ? "0000" : (line.length + 4).toString(16).padStart(4, "0") + line))
    .join("");
  return new TextEncoder().encode(body + "PACK");
}

describe("readPush and refusePush", () => {
  it("allows a recorded push to the sandbox's own branch", () => {
    const push = readPush(recorded("own"));
    expect(push).toEqual({ ok: true, updates: [{ old: Z, new: A, ref: OWN }] });
    expect(refusePush(push, OWN)).toBeNull();
  });

  it("refuses a recorded push to main", () => {
    expect(refusePush(readPush(recorded("main")), OWN)).toMatch(/not refs\/heads\/main/);
  });

  it("refuses a recorded delete", () => {
    expect(refusePush(readPush(recorded("deleteOther")), "refs/heads/other")).toMatch(/may not be deleted/);
  });

  it("refuses when any of several refs is another branch", () => {
    const body = pkt(`${Z} ${A} ${OWN}\0 report-status-v2\n`, `${Z} ${A} refs/heads/main\n`, "");
    expect(refusePush(readPush(body), OWN)).toMatch(/refs\/heads\/main/);
  });

  it("skips shallow lines from a shallow clone", () => {
    const body = pkt(`shallow ${A}\n`, `${Z} ${A} ${OWN}\0 report-status-v2\n`, "");
    expect(refusePush(readPush(body), OWN)).toBeNull();
  });

  it("refuses signed pushes, empty and cut-off bodies", () => {
    expect(refusePush(readPush(pkt("push-cert\0 report-status\n", "")), OWN)).toMatch(/signed/);
    expect(refusePush(readPush(pkt("")), OWN)).toMatch(/no ref updates/);
    expect(refusePush(readPush(new TextEncoder().encode("00c0abc")), OWN)).toMatch(/length/);
    expect(refusePush(readPush(new Uint8Array()), OWN)).toMatch(/ended/);
  });

  it("refuses a branch whose name only starts like the allowed one", () => {
    const body = pkt(`${Z} ${A} ${OWN}x\0 report-status-v2\n`, "");
    expect(refusePush(readPush(body), OWN)).not.toBeNull();
  });
});
