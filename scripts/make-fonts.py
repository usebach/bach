"""Decodes the web app's InterVariable.woff2 into the static TTF instances the
setup runtime embeds (crates/setup/assets/fonts).

    python -m pip install fonttools brotli
    python scripts/make-fonts.py path/to/InterVariable.woff2 crates/setup/assets/fonts

DirectWrite's in-memory font loader (what GPUI uses on Windows) can't read
WOFF2, and named instances of a variable font resolve unreliably through it,
so each weight the spec uses becomes its own plain TTF with clean names:
"Inter" 400/500/600 at the text optical size (opsz 14) and "Inter Display"
500 at opsz 28 for the 28px home heading, matching what the browser's
automatic optical sizing picks.
"""

import sys

from fontTools.ttLib import TTFont
from fontTools.varLib import instancer

src, out = sys.argv[1], sys.argv[2]
specs = [
  ("Inter", "Regular", 400, 14, "Inter-Regular"),
  ("Inter", "Medium", 500, 14, "Inter-Medium"),
  ("Inter", "SemiBold", 600, 14, "Inter-SemiBold"),
  ("Inter Display", "Medium", 500, 28, "InterDisplay-Medium"),
]
for fam, style, w, opsz, ps in specs:
    f = TTFont(src)
    inst = instancer.instantiateVariableFont(f, {"wght": w, "opsz": opsz})
    name = inst['name']
    # keep only the names we set
    name.names = [n for n in name.names if n.nameID not in (1,2,3,4,6,16,17,21,22,25)]
    legacy_family = fam if style == "Regular" else f"{fam} {style}"
    for pid, eid, lid in ((3,1,0x409),):
        name.setName(legacy_family, 1, pid, eid, lid)
        name.setName("Regular", 2, pid, eid, lid)
        name.setName(f"{ps};whirl", 3, pid, eid, lid)
        name.setName(f"{fam} {style}", 4, pid, eid, lid)
        name.setName(ps, 6, pid, eid, lid)
        name.setName(fam, 16, pid, eid, lid)
        name.setName(style, 17, pid, eid, lid)
    inst['OS/2'].usWeightClass = w
    # RIBBI bits: regular only for 400
    inst['OS/2'].fsSelection = (inst['OS/2'].fsSelection & ~0b1100001) | (0b1000000 if w == 400 else 0)
    inst['head'].macStyle = 0
    for t in ('STAT',):
        if t in inst: del inst[t]
    inst.flavor = None
    inst.save(f"{out}/{ps}.ttf")
    print(ps, inst['OS/2'].usWeightClass)
