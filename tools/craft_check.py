#!/usr/bin/env python3
"""Craft every carried elevatorid recipe in a real crafting table.

Called by `tools/e2e.sh` with an open game's control socket (the leg launches `--offscreen`,
because `click-slot` needs a renderer). The 16 colours carry 16 recipes, carried from
ElevatorMod 1.21.1-1.11.4:

* `elevator_white` — shaped, eight `minecraft:white_wool` in the ring and one
  `minecraft:ender_pearl` in the centre (`data/elevatorid/recipe/elevator_white.json`);
* `redye_<colour>` — shapeless, one elevator of the `#elevatorid:elevators` tag and one of the
  `#c:dyes/<colour>` tag, for the other fifteen (`data/elevatorid/recipe/redye_*.json`).

Each case gives the declared ingredients (hotbar slot 0 and 1 after `clear @s`), opens a fresh
crafting table, places them the way the recipe declares (shaped: the ring and the centre;
shapeless: two grid cells), takes the result, and asserts the **output id and count** and that the
**ingredients were consumed** — the inputs gone from the inventory once the result is taken.

The last case is a control that must craft nothing: eight wool without the ender pearl is not the
shaped pattern, so the shaped assertion discriminates the pattern rather than "an elevator can be
made somehow".

Menu slots (the harness's crafting-table numbering): grid 100-108 row-major, result 200. The
inventory is read through `ctl inventory`; no `ctl screen`/`ctl ui` (they need a renderer and are
slower).

Exits 1 with the observed result on the first mismatch.
"""
import json
import subprocess
import sys

RING = [100, 101, 102, 103, 105, 106, 107, 108]
CENTRE = 104
RESULT = 200

COLOURS = [
    "orange", "magenta", "light_blue", "yellow", "lime", "pink", "gray", "light_gray",
    "cyan", "purple", "blue", "brown", "green", "red", "black",
]


def main():
    if len(sys.argv) != 6:
        raise SystemExit("usage: craft_check.py SOCK CTL X Y Z")
    sock, ctl = sys.argv[1], sys.argv[2]
    x, y, z = int(sys.argv[3]), int(sys.argv[4]), int(sys.argv[5])

    def run(*args):
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"ctl {' '.join(args)}: {out.stdout}{out.stderr}")

    def wait(ticks=4):
        run("wait", str(ticks))

    def counts():
        return run("inventory").get("counts", {})

    def setup(items):
        run("close")
        wait(3)
        # A fresh table per case: its block entity holds the grid, so nothing leaks between
        # cases (a failed or untaken result would otherwise change the next grid).
        run("setblock", str(x), str(y), str(z), "minecraft:air")
        run("setblock", str(x), str(y), str(z), "minecraft:crafting_table")
        wait(2)
        run("command", f"tp @s {x - 0.5} {y} {z + 0.5}")
        wait(3)
        run("command", "clear @s")
        for item, count in items:
            run("give", item, str(count))
        wait(3)
        reply = run("interact", str(x), str(y), str(z))
        if reply.get("screen") not in ("crafting_table", "inventory", "unknown"):
            raise SystemExit(f"the crafting table did not open: {reply}")
        wait(5)

    def place(source, mode, targets):
        run("click-slot", str(source))
        wait(1)
        for target in targets:
            if mode == "right":
                run("click-slot", str(target), "right")
            else:
                run("click-slot", str(target))
            wait(1)
        # Whatever the cursor still holds goes back where it came from.
        run("click-slot", str(source))
        wait(2)

    def take_result():
        reply = run("click-slot", str(RESULT) + " shift")
        wait(2)
        return reply

    failures = []
    crafted = 0

    def check(label, items, placements, want_item, want_count, inputs):
        nonlocal crafted
        setup(items)
        for source, mode, targets in placements:
            place(source, mode, targets)
        take_result()
        run("close")
        wait(3)
        tally = counts()
        left = {item: tally.get(item, 0) for item in inputs}
        got = tally.get(want_item, 0)
        print(f"{label}: inventory -> {json.dumps({want_item: got, **left})}")
        if any(left.values()):
            failures.append(f"{label}: ingredients were not consumed: {left}")
            return
        if got < want_count:
            failures.append(f"{label}: {want_item}={got} in the inventory, want {want_count}")
            return
        crafted += 1

    # 1. The shaped white recipe: the ring of eight wool and the pearl in the centre.
    #    Hotbar slot 0 holds the wool, slot 1 the pearl (after `clear @s`).
    check(
        "elevator_white (shaped ring)",
        [("minecraft:white_wool", 8), ("minecraft:ender_pearl", 1)],
        [(0, "right", RING), (1, "left", [CENTRE])],
        "elevatorid:elevator_white",
        1,
        ["minecraft:white_wool", "minecraft:ender_pearl"],
    )

    # 2. Every redye: one elevator of the tag plus one dye of `#c:dyes/<colour>`.
    for colour in COLOURS:
        dye = f"minecraft:{colour}_dye"
        check(
            f"{colour} (shapeless redye)",
            [("elevatorid:elevator_white", 1), (dye, 1)],
            [(0, "left", [100]), (1, "left", [102])],
            f"elevatorid:elevator_{colour}",
            1,
            ["elevatorid:elevator_white", dye],
        )

    # 3. The control: no pearl, so the shaped pattern is incomplete and nothing is crafted.
    setup([("minecraft:white_wool", 8)])
    place(0, "right", RING)
    take_result()
    run("close")
    wait(3)
    tally = counts()
    print(f"eight wool without the pearl: inventory -> {json.dumps({k: v for k, v in tally.items() if 'elevator' in k} or {})}")
    if tally.get("elevatorid:elevator_white", 0):
        failures.append("the shaped recipe crafted from eight wool and no pearl")
    if tally.get("minecraft:white_wool", 0) != 8:
        failures.append(f"the control's eight wool were not all returned: {tally.get('minecraft:white_wool', 0)}")

    if failures:
        for failure in failures:
            print(f"FAIL: {failure}", file=sys.stderr)
        raise SystemExit(1)
    print(f"PASS: {crafted}/16 recipes crafted, each consumed its ingredients; eight wool alone is not the pattern")


if __name__ == "__main__":
    main()
