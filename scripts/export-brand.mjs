// Run with `bun scripts/export-brand.mjs` after editing the SVGs in brand/.
// GPUI can't play SVG CSS animations, so export the four distinct petals in
// square boxes centred on their fill bounds. BachMark replays the flip.
// Copy the printed geometry into PETALS in ui/bach_mark.rs when it changes.

import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";

const brand = new URL("../brand/", import.meta.url);
const assets = new URL("../crates/setup/assets/", import.meta.url);
const source = readFileSync(new URL("bach-mark-animated.svg", brand), "utf8");
const petals = [...source.matchAll(/<path class="p" style="animation-delay:([\d.]+)s" d="([^"]+)"/g)]
  .sort(([, a], [, b]) => Number(a) - Number(b))
  .map(([, , path]) => path);
const shapes = [...new Set(petals)];
if (petals.length !== 12 || shapes.length !== 4 || petals.some((path, index) => path !== shapes[index % 4])) {
  throw new Error("Expected four petal shapes repeated three times clockwise from the top.");
}

// The rounded ends are circular arcs with vertically aligned endpoints.
// Each circle centre lies to the right of its endpoints in local SVG space.
const bounds = shapes.map((path) => {
  const arcs = [...path.matchAll(/([\d.]+) (-?[\d.]+)A([\d.]+) [\d.]+ 0 [01] 0 [\d.]+ -?[\d.]+/g)];
  if (arcs.length !== 2) throw new Error("Expected two circular ends per petal.");
  const [outer, inner] = arcs.map(([, x, y, radius]) => {
    const r = Number(radius);
    return { center: Number(x) + Math.sqrt(r ** 2 - Number(y) ** 2), radius: r };
  });
  const left = inner.center - inner.radius;
  const right = outer.center + outer.radius;
  return { center: (left + right) / 2, side: Math.ceil(right - left) + 2 };
});

copyFileSync(new URL("bach-mark.svg", brand), new URL("icons/bach.svg", assets));
mkdirSync(new URL("bach/", assets), { recursive: true });
for (const [index, { center, side }] of bounds.entries()) {
  const left = (center - side / 2).toFixed(3);
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${side}" height="${side}" viewBox="${left} ${-side / 2} ${side} ${side}">
  <path d="${shapes[index]}"/>
</svg>\n`;
  writeFileSync(new URL(`bach/petal-${index + 1}.svg`, assets), svg);
  console.log(`    Petal { distance: ${center.toFixed(3)}, side: ${side}.0 },`);
}
