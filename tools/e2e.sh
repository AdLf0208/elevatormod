#!/usr/bin/env bash
# In-game check for ElevatorMod 0.2.0 (jump up / sneak down).
#
# STATUS (2026-11-18, SDK migration): this scenario needs a game the **launcher** built with this
# flat SDK package — a native mod is compiled into `horizon-client`:
#
#   horizon-launcher native-build ports/elevatorid --instance <i> --trust-native-code
#
# then point `GAME=` at that build. `horizon-launcher` is not installed in this environment and no
# game exists at the pinned SDK (`0b14f8a`) here, so this migration did **not** re-run it; the
# 0.1.0 runs under `../_shared/evidence/elevatorid/` (e2e.log, e2e-selftest.log, e2e-control.log)
# are historical evidence of the same behaviour from the retired `.hnmod`, and `notes.md` records
# the missing run as a gap. The scenario and its assertions are format-independent.
#
# The flat package carries `assets/` and `data/` itself, so there is no `--dev-mod` and no embedded
# content to check; the elevators are simply blocks of the installed mod.
#
# The leg launches `--offscreen`, not `--headless`: the recipe phase drives a crafting table with
# `ctl click-slot`, which answers "no renderer yet" without one. That also keeps the leg local —
# the far build box has no renderer (`ports/_shared/remote-build.md`).
#
# The End, stone floor at y=63 (feet 64), elevators at y=63/66/69 in one column.
#  - jump: nearest target 66 -> feet y=67
#  - sneak: down to 63 -> feet y=64
#  - plain floor jump / sneak: no travel
#  - lone elevator: no target above -> normal jump, lands back
#  - stone above the middle elevator: jump skips to 69 -> feet y=70
#  - a second column (x=3) for the arrival view: a plain target (directional=false, facing=east)
#    keeps the look (yaw 33, pitch 40); the same target with directional=true turns to yaw 270
#    (east) and levels the pitch to 0
#  - a crafting table at 6,64,6: `tools/craft_check.py` crafts the shaped white recipe and all
#    fifteen redyes, asserting each output id/count and that the ingredients were consumed, with
#    eight wool and no pearl as the control that must find no result
#
# CONTROL_GAME=<an unmodified build> adds the differential leg: the same scene on a game without
# the mod has no `elevatorid:` blocks at all, so the jump stays at y=64.
# SELFTEST=1 keeps the old body-predicate expectation (64) for the blocked-target step, so the run
# must FAIL: the proof the assertion can fail.
#
# 0.2.2 adds the arrival view (upstream's `directional`/`facing`): a directional target turns the
# player to its facing and levels the pitch, a plain one keeps the view it arrived with — the same
# block, same facing value, only `directional` flipped, so the pair is its own differential.
# SELFTEST=look keeps the pre-0.2.2 view-kept expectation for the directional step and must FAIL.
set -euo pipefail

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/elevatorid
# The control client must speak the same protocol as GAME. No shared binary exists at the pinned
# SDK yet (`ports/_shared/bin/horizon-0b14f8ac/` is not published), so build one from the same
# Horizon revision as GAME, or point CTL at it.
CTL=${CTL:-$CLONE/ports/_shared/bin/horizon-0b14f8ac/horizon-ctl}
GAME=${GAME:-$PORT/target/horizon-cargo/fast/horizon-client}
CONTROL_GAME=${CONTROL_GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/elevatorid-e2e.sock}
SELFTEST=${SELFTEST:-0}

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client (a launcher build with this mod): $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d /tmp/opencode/elevatorid-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
fail() { echo "FAIL: $*" >&2; exit 1; }
feet_y() { ctl state | jq -r '.player.position[1]'; }
yaw() { ctl state | jq -r '.player.yaw'; }
pitch() { ctl state | jq -r '.player.pitch'; }
near() { awk -v a="$1" -v b="$2" 'BEGIN { d = a - b; if (d < 0) d = -d; exit !(d < 0.6) }'; }
# Two angles are the same turn when they differ by less than a degree, either spelling of the
# wrap (-90 is 270).
angle_near() {
    awk -v a="$1" -v b="$2" 'BEGIN { d = (a - b) % 360; if (d < 0) d += 360; if (d > 180) d = 360 - d; exit !(d < 1.0) }'
}

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    rm -rf "$SCRATCH" 2>/dev/null || true
    echo "EXIT:$status"
}
trap cleanup EXIT

launch() { # launch <game> <world>
    cd "$SCRATCH"
    "$CTL" --socket "$SOCK" launch \
        --game "$1" --world "$2" --seed 20261102 --creative --offscreen --wait \
        --resources "$RES"
    ctl wait-ready 120
}

# The scene: floor and the three elevators, the player on the bottom one.
scene() {
    ctl command "execute in minecraft:the_end run tp @s 0.5 64 0.5"
    ctl wait 10
    ctl command "execute in minecraft:the_end run fill -12 63 -12 12 63 12 minecraft:stone"
    ctl command "execute in minecraft:the_end run fill -12 64 -12 12 72 12 minecraft:air"
    ctl command "execute in minecraft:the_end run kill @e[type=!player]"
    ctl command "execute in minecraft:the_end run setblock 0 63 0 elevatorid:elevator_white"
    ctl command "execute in minecraft:the_end run setblock 0 66 0 elevatorid:elevator_white"
    ctl command "execute in minecraft:the_end run setblock 0 69 0 elevatorid:elevator_white"
    ctl command "execute in minecraft:the_end run tp @s 0.5 64 0.5"
    # The End's exit portal puts a dragon egg at 0,64,0 when the player arrives; clear the column
    # so the scene is the elevator blocks and air.
    ctl command "execute in minecraft:the_end run setblock 0 64 0 minecraft:air"
    ctl command "execute in minecraft:the_end run setblock 0 65 0 minecraft:air"
    ctl wait 5
}

echo "== launch (The End, launcher-built game carrying the mod) =="
launch "$GAME" ElevatorE2E
scene

echo "== jump: nearest elevator above is 66 =="
ctl jump
ctl wait 10
y=$(feet_y); echo "after jump: y=$y"
near "$y" 67 || fail "expected y=67 after jump, got $y"

echo "== sneak: down to 63 =="
ctl hold shift 0.2
ctl wait 10
y=$(feet_y); echo "after sneak: y=$y"
near "$y" 64 || fail "expected y=64 after sneak, got $y"

echo "== control: plain floor, no elevator =="
ctl command "execute in minecraft:the_end run tp @s 8.5 64 0.5"
ctl wait 5
ctl jump
ctl wait 10
y=$(feet_y); echo "plain-floor jump: y=$y"
near "$y" 64 || fail "a plain-floor jump travelled (y=$y)"
ctl hold shift 0.2
ctl wait 10
y=$(feet_y); echo "plain-floor sneak: y=$y"
near "$y" 64 || fail "a plain-floor sneak travelled (y=$y)"

echo "== control: lone elevator has no target above =="
ctl command "execute in minecraft:the_end run setblock 6 63 0 elevatorid:elevator_red"
ctl command "execute in minecraft:the_end run tp @s 6.5 64 0.5"
ctl wait 5
ctl jump
ctl wait 10
y=$(feet_y); echo "lone-elevator jump: y=$y"
near "$y" 64 || fail "a lone elevator travelled (y=$y)"

echo "== obstruction: stone above 66 makes the jump skip to 69 =="
ctl command "execute in minecraft:the_end run setblock 0 67 0 minecraft:stone"
ctl command "execute in minecraft:the_end run tp @s 0.5 64 0.5"
ctl wait 5
ctl jump
ctl wait 10
y=$(feet_y); echo "blocked-target jump: y=$y"
# The recorded failing case of the 0.1.0 body predicate: a jump clipped under the 2-block ceiling
# reported a fall, so the run stayed at 64. SELFTEST=1 keeps that old expectation and must fail,
# proving the assertion is not vacuous.
expected=70
[ "$SELFTEST" = 1 ] && expected=64
near "$y" "$expected" || fail "expected y=$expected (skipping the blocked 66), got $y"

echo "== colour: a red landing at 66 is skipped, the white one at 69 is taken =="
ctl command "execute in minecraft:the_end run setblock 0 67 0 minecraft:air"
ctl command "execute in minecraft:the_end run setblock 0 66 0 elevatorid:elevator_red"
ctl command "execute in minecraft:the_end run tp @s 0.5 64 0.5"
ctl wait 5
ctl jump
ctl wait 10
y=$(feet_y); echo "same-colour jump past red at 66: y=$y"
near "$y" 70 || fail "expected y=70 (the white elevator at 69), got $y"

echo "== colour control: no same-colour target above means no travel =="
ctl command "execute in minecraft:the_end run setblock 0 69 0 elevatorid:elevator_red"
ctl command "execute in minecraft:the_end run tp @s 0.5 64 0.5"
ctl wait 5
ctl jump
ctl wait 10
y=$(feet_y); echo "all-red-above jump: y=$y"
near "$y" 64 || fail "a jump landed on another colour's elevator (y=$y)"

echo "== arrival view: a plain elevator keeps the view it arrived with =="
ctl command "execute in minecraft:the_end run setblock 3 63 0 elevatorid:elevator_white"
ctl command "execute in minecraft:the_end run setblock 3 66 0 elevatorid:elevator_white[directional=false,facing=east]"
ctl wait 5
state=$(ctl block 3 66 0 | jq -r '.state')
echo "plain target state: $state"
case "$state" in *"facing=east"*) ;; *) fail "the plain target lost its declared facing: $state" ;; esac
case "$state" in *"directional=false"*) ;; *) fail "the plain target is not directional=false: $state" ;; esac
ctl command "execute in minecraft:the_end run tp @s 3.5 64 0.5"
ctl wait 5
ctl look 33 40
ctl wait 5
ctl jump
ctl wait 10
y=$(feet_y); a=$(yaw); p=$(pitch)
echo "plain jump: y=$y yaw=$a pitch=$p"
near "$y" 67 || fail "expected y=67 on the plain elevator, got $y"
angle_near "$a" 33 || fail "a plain elevator turned the player (yaw=$a, expected 33 kept)"
near "$p" 40 || fail "a plain elevator levelled the pitch (pitch=$p, expected 40 kept)"

echo "== arrival view: a directional elevator turns you to its facing and levels the pitch =="
ctl command "execute in minecraft:the_end run setblock 3 66 0 elevatorid:elevator_white[directional=true,facing=east]"
ctl wait 5
state=$(ctl block 3 66 0 | jq -r '.state')
echo "directional target state: $state"
case "$state" in *"facing=east"*) ;; *) fail "the directional target lost facing=east: $state" ;; esac
case "$state" in *"directional=true"*) ;; *) fail "the directional target is not directional=true: $state" ;; esac
ctl command "execute in minecraft:the_end run tp @s 3.5 64 0.5"
ctl wait 5
ctl look 33 40
ctl wait 5
ctl jump
ctl wait 10
y=$(feet_y); a=$(yaw); p=$(pitch)
# Same block, same facing value, only `directional` flipped from the plain phase above, so the
# two phases are each other's differential. SELFTEST=look keeps the pre-0.2.2 view-kept
# expectation and must fail, proving this assertion is not vacuous.
expected_yaw=270; expected_pitch=0
if [ "$SELFTEST" = look ]; then expected_yaw=33; expected_pitch=40; fi
echo "directional jump: y=$y yaw=$a pitch=$p (expected yaw=$expected_yaw pitch=$expected_pitch)"
near "$y" 67 || fail "expected y=67 on the directional elevator, got $y"
angle_near "$a" "$expected_yaw" || fail "expected yaw=$expected_yaw after a directional ride, got $a"
near "$p" "$expected_pitch" || fail "expected pitch=$expected_pitch after a directional ride, got $p"

echo "== recipes: the shaped white recipe and all fifteen redyes craft and consume their inputs =="
ctl command "execute in minecraft:the_end run tp @s 6.5 64 6.5"
ctl wait 5
python3 "$PORT/tools/craft_check.py" "$SOCK" "$CTL" 6 64 6 || fail "the recipe check failed"
ctl command "execute in minecraft:the_end run setblock 6 64 6 minecraft:air"
ctl command "clear @s"
ctl wait 5

if [ -n "$CONTROL_GAME" ]; then
    [ -x "$CONTROL_GAME" ] || fail "missing control game: $CONTROL_GAME"
    ctl quit >/dev/null 2>&1 || true
    rm -f "$SOCK"
    launch "$CONTROL_GAME" ElevatorE2E-control
    scene
    ctl jump
    # A normal jump has no ride to carry it: wait for the arc to land before reading.
    landed=false
    for _ in $(seq 1 40); do
        ctl wait 2 >/dev/null
        y=$(feet_y)
        if [ "$(ctl state | jq -r '.player.on_ground')" = true ]; then landed=true; break; fi
    done
    echo "unmodified-game landed=$landed y=$y block64=$(ctl block 0 64 0 | jq -r '.name // .block') block65=$(ctl block 0 65 0 | jq -r '.name // .block')"
    # No elevator ids exist there, so the setblocks did nothing and the jump cannot travel.
    near "$y" 64 || fail "a jump travelled without the mod (y=$y)"
fi

ctl quit >/dev/null 2>&1 || true
trap - EXIT
sleep 1
rm -rf "$SCRATCH" 2>/dev/null || true
echo "PASS: jump 67, sneak 64, plain floor stayed, lone elevator stayed, blocked target skipped to 70, another colour's landing skipped and only a same-colour one taken, plain arrival kept the view and directional arrival turned to the facing with a level pitch, 16/16 recipes crafted and consumed their inputs"
[ -n "$CONTROL_GAME" ] && echo "PASS: differential on an unmodified game stayed at 64"
echo "EXIT:0"
