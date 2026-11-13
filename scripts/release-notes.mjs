// Writes the changelog for a release from the commits since the last tag.
//
//   bun scripts/release-notes.mjs --version 0.2.0            # print the section
//   bun scripts/release-notes.mjs --version 0.2.0 --write    # also update files
//
// --write puts the section at the top of CHANGELOG.md and writes
// dist/release-notes.md (the section plus install notes) for the GitHub
// release. Commits are sorted by the verb they start with, the way this
// repository writes subjects: "Add …" is new, "Fix …" is a fix, the rest
// are improvements. Merges and earlier "Release v…" commits are left out.

import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const root = join(import.meta.dir, "..");
const args = process.argv.slice(2);
const version = valueOf("--version");
const write = args.includes("--write");

function valueOf(flag) {
  const index = args.indexOf(flag);
  return index >= 0 ? args[index + 1] : undefined;
}

function git(...gitArgs) {
  return execFileSync("git", gitArgs, { cwd: root, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] }).trim();
}

if (!version) {
  console.error("Say which version these notes are for: --version 0.2.0");
  process.exit(1);
}

/** https://github.com/owner/repo, from the Actions environment or the remote. */
function repositoryUrl() {
  if (process.env.GITHUB_REPOSITORY) {
    return `${process.env.GITHUB_SERVER_URL ?? "https://github.com"}/${process.env.GITHUB_REPOSITORY}`;
  }
  try {
    const remote = git("remote", "get-url", "origin");
    const match = /github\.com[:/](.+?)(?:\.git)?$/.exec(remote);
    return match ? `https://github.com/${match[1]}` : null;
  } catch {
    return null;
  }
}

function lastTag() {
  try {
    return git("describe", "--tags", "--abbrev=0", "--match", "v*");
  } catch {
    return null; // The first release: everything so far.
  }
}

const NEW = /^(add|introduce|support|implement|create|allow|let|show|offer)\b/i;
const FIX = /^(fix|repair|correct|prevent|avoid|restore|handle|stop)\b/i;

const previous = lastTag();
const range = previous ? `${previous}..HEAD` : "HEAD";
const log = git("log", range, "--no-merges", "--format=%H%x1f%s%x1e");
const commits = log
  .split("\x1e")
  .map((entry) => entry.trim())
  .filter(Boolean)
  .map((entry) => {
    const [hash, subject] = entry.split("\x1f");
    return { hash, subject: subject.trim() };
  })
  .filter(({ subject }) => !/^release v?\d/i.test(subject));

const url = repositoryUrl();
const groups = [
  ["New", commits.filter(({ subject }) => NEW.test(subject))],
  ["Fixes", commits.filter(({ subject }) => FIX.test(subject))],
  ["Improvements", commits.filter(({ subject }) => !NEW.test(subject) && !FIX.test(subject))],
];

function line({ hash, subject }) {
  const short = hash.slice(0, 7);
  const link = url ? `[${short}](${url}/commit/${hash})` : short;
  return `- ${subject} (${link})`;
}

const date = new Date().toISOString().slice(0, 10);
const parts = [`## ${version} (${date})`, ""];
for (const [title, items] of groups) {
  if (items.length === 0) continue;
  parts.push(`### ${title}`, "", ...items.map(line), "");
}
if (commits.length === 0) parts.push("No changes since the last release.", "");
if (previous && url) parts.push(`All changes: [${previous}...v${version}](${url}/compare/${previous}...v${version})`, "");
const section = parts.join("\n");

if (!write) {
  console.log(section);
  process.exit(0);
}

const changelogPath = join(root, "CHANGELOG.md");
const header =
  "# Changelog\n\nWhat changed in each release of Bach, newest first. The release workflow writes this from the commit history.\n\n";
const existing = existsSync(changelogPath) ? readFileSync(changelogPath, "utf8").replace(/\r\n/g, "\n") : header;
const body = existing.startsWith(header) ? existing.slice(header.length) : existing.replace(/^# Changelog\n+/, "");
writeFileSync(changelogPath, `${header}${section}\n${body}`.replace(/\n{3,}/g, "\n\n"));

const install = [
  "### Install",
  "",
  `Download **bach-${version}-windows-x64.zip** below and unzip it. Keep \`bach.exe\` and \`bach-setup.exe\` together, then run \`bach init\` in your project and \`bach build\`. The README inside walks through it.`,
  "",
  "The binaries aren't code-signed yet, so Windows SmartScreen may warn the first time you run an installer you built: choose **More info → Run anyway**, or sign your setup file (see docs/signing.md).",
  "",
].join("\n");
mkdirSync(join(root, "dist"), { recursive: true });
// The release page has its own title, so the notes start below the heading.
const notes = section.split("\n").slice(2).join("\n");
writeFileSync(join(root, "dist", "release-notes.md"), `${notes}\n${install}`);
console.log(section);
