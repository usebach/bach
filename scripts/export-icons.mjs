// Exports Tabler icons from the @tabler/icons-react package as SVG files the
// setup runtime embeds, so the installer window uses Bach's usual glyphs.
//
//   bun install
//   bun scripts/export-icons.mjs IconFolderFilled IconArrowUp IconPlus@2.5 ...
//
// Writes crates/setup/assets/icons/<kebab-name>.svg. `Name@2.5` exports an
// outline icon at another stroke width (the web's `stroke={2.5}`) as
// <kebab-name>@2.5.svg, which `ui::Icon::new("plus").stroke(2.5)` picks up.
// Outline icons keep Tabler's round strokes; filled icons are solid. GPUI
// renders them as a mask and colours them with the element's text colour.

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const iconsDir = join(here, "../node_modules/@tabler/icons-react/dist/esm/icons");
const outDir = join(here, "../crates/setup/assets/icons");

const names = process.argv.slice(2);
if (names.length === 0) {
  console.error("Usage: node scripts/export-icons.mjs IconName [IconName ...]");
  process.exit(1);
}

mkdirSync(outDir, { recursive: true });

function attributes(attrs) {
  return Object.entries(attrs)
    .filter(([key]) => key !== "key")
    .map(([key, value]) => `${key}="${String(value).replace(/"/g, "&quot;")}"`)
    .join(" ");
}

for (const arg of names) {
  const [name, stroke = "2"] = arg.split("@");
  const source = readFileSync(join(iconsDir, `${name}.mjs`), "utf8");
  const node = JSON.parse(/const __iconNode = (\[.*\]);/.exec(source)[1]);
  const [, type, kebab] = /createReactComponent\("(\w+)", "([\w-]+)"/.exec(source);

  const root =
    type === "filled"
      ? 'fill="currentColor" stroke="none"'
      : `fill="none" stroke="currentColor" stroke-width="${stroke}" stroke-linecap="round" stroke-linejoin="round"`;
  const children = node.map(([tag, attrs]) => `<${tag} ${attributes(attrs)}/>`).join("");
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" ${root}>${children}</svg>\n`;

  const file = stroke === "2" ? `${kebab}.svg` : `${kebab}@${stroke}.svg`;
  writeFileSync(join(outDir, file), svg);
  console.log(`${arg} -> assets/icons/${file}`);
}
