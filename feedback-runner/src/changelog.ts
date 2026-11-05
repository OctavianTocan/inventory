// Reads and renumbers the top entry of android/CHANGELOG.md, with the rules `history()` in deploy/android-release.py
// applies, so a renumbered file still passes the release gate.

const HEADING = /^## (\d+)\.(\d+)\.(\d+) \((\d+)\), (\d{4}-\d{2}-\d{2})$/;

/** The version of one CHANGELOG entry. */
export interface Version {
  name: string;
  code: number;
}

/** The first `## X.Y.Z (code), date` heading of a CHANGELOG, or null when there is none. */
export function topVersion(text: string): Version | null {
  for (const line of text.split("\n")) {
    const match = HEADING.exec(line.trimEnd());
    if (match) return { name: `${match[1]}.${match[2]}.${match[3]}`, code: Number(match[4]) };
  }
  return null;
}

/** Compares two `X.Y.Z` names; negative when `a` is older. */
export function compareNames(a: string, b: string): number {
  const [x, y] = [a.split(".").map(Number), b.split(".").map(Number)];
  for (let i = 0; i < 3; i++) if (x[i] !== y[i]) return x[i] - y[i];
  return 0;
}

/** The version after `version`: patch and code both one higher. */
export function nextVersion(version: Version): Version {
  const [major, minor, patch] = version.name.split(".").map(Number);
  return { name: `${major}.${minor}.${patch + 1}`, code: version.code + 1 };
}

/**
 * The entry a branch added on top of the CHANGELOG it started from: its heading line through the line before the next
 * heading, or null when its top entry is one `main` already had.
 *
 * @param branch the branch's CHANGELOG
 * @param started `main`'s CHANGELOG at the commit the branch started from
 * @returns the entry's lines, heading first
 */
export function ownEntry(branch: string, started: string): string[] | null {
  const ours = topVersion(branch);
  const base = topVersion(started);
  if (!ours || !base || ours.code <= base.code) return null;
  const lines = branch.split("\n");
  const first = lines.findIndex((line) => HEADING.test(line.trimEnd()));
  let end = first + 1;
  while (end < lines.length && !lines[end].startsWith("## ")) end++;
  return lines.slice(first, end);
}

/**
 * `main`'s CHANGELOG with a branch's own entry put back on top, numbered one above `main`'s top entry.
 *
 * The merge queue takes the entry out before it merges `main` into the branch, because two branches that each add a
 * top entry always conflict, then puts it back with this. Only the numbers in the heading change; date and notes stay.
 *
 * @param main `main`'s CHANGELOG
 * @param entry the branch's entry from `ownEntry`
 * @returns the new text and the entry's version
 */
export function withEntry(main: string, entry: string[]): { text: string; version: Version } {
  const theirs = topVersion(main);
  const heading = HEADING.exec(entry[0].trimEnd());
  if (!theirs || !heading) throw new Error("CHANGELOG has no release entry");
  const version = nextVersion(theirs);
  const block = [`## ${version.name} (${version.code}), ${heading[5]}`, ...entry.slice(1)];
  while (block.length > 1 && block[block.length - 1].trim() === "") block.pop();
  const lines = main.split("\n");
  const first = lines.findIndex((line) => HEADING.test(line.trimEnd()));
  return { text: [...lines.slice(0, first), ...block, "", ...lines.slice(first)].join("\n"), version };
}
