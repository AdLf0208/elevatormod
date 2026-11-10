#!/usr/bin/env bash
# In-game check for ElevatorMod slice 1 (jump up / sneak down).
#
# The End, stone floor at y=63 (feet 64), elevators at y=63/66/69 in one column.
#  - jump: nearest target 66 → feet y=67
#  - sneak: down to 63 → feet y=64
#  - plain floor jump / sneak: no travel
#  - lone elevator: no target above → normal jump, lands back
#  - stone above the middle elevator: jump skips to 69 → feet y=70
set -euo pipefail

CLONE=${CAMPAIGN_ROOT:-$(cd "$(dirname "$0")/../../.." && pwd)}
PORT=$CLONE/ports/elevatorid
CTL=${CTL:-$CLONE/ports/_shared/bin/horizon-1d63e0a5/horizon-ctl}
GAME=${GAME:-$PORT/target/horizon-cargo/fast/horizon-client}
RES=${RES:-${HORIZON_RES:-${HORIZON_RES:-$HOME/.local/horizon-client/26.3}}}
SOCK=${SOCK:-/tmp/opencode/elevatorid-e2e.sock}
SELFTEST=${SELFTEST:-0}
# EMBEDDED=1 launches the native bundle alone (no --dev-mod).
EMBEDDED=${EMBEDDED:-0}
DEV_ARGS=()
[ "$EMBEDDED" = 1 ] || DEV_ARGS=(--dev-mod "$PORT/content")

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
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
near() { awk -v a="$1" -v b="$2" 'BEGIN { d = a - b; if (d < 0) d = -d; exit !(d < 0.6) }'; }

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    rm -rf "$SCRATCH" 2>/dev/null || true
    echo "EXIT:$status"
}
trap cleanup EXIT

echo "== launch (The End, content --dev-mod) =="
cd "$SCRATCH"
"$CTL" --socket "$SOCK" launch \
    --game "$GAME" --world ElevatorE2E --seed 20261102 --creative --headless --wait \
    --resources "$RES" "${DEV_ARGS[@]}"
ctl wait-ready 120
if [ "$EMBEDDED" = 1 ]; then
    cache=$SCRATCH/.horizon-cache/elevatorid-embedded-content.hmod
    [ -f "$cache" ] || fail "the bundle did not write its embedded content to $cache"
    cmp -s "$cache" "$PORT/content/elevatorid.hmod" \
        || fail "the embedded content differs from content/elevatorid.hmod"
    echo "  embedded content: $(wc -c <"$cache" | tr -d ' ') B, identical to content/elevatorid.hmod"
fi
ctl command "execute in minecraft:the_end run tp @s 0.5 64 0.5"
ctl wait 10
ctl command "execute in minecraft:the_end run fill -12 63 -12 12 63 12 minecraft:stone"
ctl command "execute in minecraft:the_end run fill -12 64 -12 12 72 12 minecraft:air"
ctl command "execute in minecraft:the_end run kill @e[type=!player]"
ctl command "execute in minecraft:the_end run setblock 0 63 0 elevatorid:elevator_white"
ctl command "execute in minecraft:the_end run setblock 0 66 0 elevatorid:elevator_white"
ctl command "execute in minecraft:the_end run setblock 0 69 0 elevatorid:elevator_white"
ctl command "execute in minecraft:the_end run tp @s 0.5 64 0.5"
ctl wait 5

echo "== jump: nearest elevator above is 66 =="
ctl jump
ctl wait 10
y=$(feet_y); echo "after jump: y=$y"
near "$y" 67 || { echo "FAIL: expected y=67 after jump, got $y" >&2; exit 1; }

echo "== sneak: down to 63 =="
ctl hold shift 0.2
ctl wait 10
y=$(feet_y); echo "after sneak: y=$y"
near "$y" 64 || { echo "FAIL: expected y=64 after sneak, got $y" >&2; exit 1; }

echo "== control: plain floor, no elevator =="
ctl command "execute in minecraft:the_end run tp @s 8.5 64 0.5"
ctl wait 5
ctl jump
ctl wait 10
y=$(feet_y); echo "plain-floor jump: y=$y"
near "$y" 64 || { echo "FAIL: a plain-floor jump travelled (y=$y)" >&2; exit 1; }
ctl hold shift 0.2
ctl wait 10
y=$(feet_y); echo "plain-floor sneak: y=$y"
near "$y" 64 || { echo "FAIL: a plain-floor sneak travelled (y=$y)" >&2; exit 1; }

echo "== control: lone elevator has no target above =="
ctl command "execute in minecraft:the_end run setblock 6 63 0 elevatorid:elevator_red"
ctl command "execute in minecraft:the_end run tp @s 6.5 64 0.5"
ctl wait 5
ctl jump
ctl wait 10
y=$(feet_y); echo "lone-elevator jump: y=$y"
near "$y" 64 || { echo "FAIL: a lone elevator travelled (y=$y)" >&2; exit 1; }

echo "== obstruction: stone above 66 makes the jump skip to 69 =="
ctl command "execute in minecraft:the_end run setblock 0 67 0 minecraft:stone"
ctl command "execute in minecraft:the_end run tp @s 0.5 64 0.5"
ctl wait 5
ctl jump
ctl wait 10
y=$(feet_y); echo "blocked-target jump: y=$y"
# The recorded failing case: the body-based take-off predicate used to report a
# fall here (a jump under the 2-block ceiling is clipped), so the run stayed at
# 64. SELFTEST=1 keeps that old expectation and must fail, proving the
# assertion is not vacuous.
expected=70
[ "$SELFTEST" = 1 ] && expected=64
near "$y" "$expected" || { echo "FAIL: expected y=$expected (skipping the blocked 66), got $y" >&2; exit 1; }

ctl quit >/dev/null 2>&1 || true
trap - EXIT
sleep 1
rm -rf "$SCRATCH" 2>/dev/null || true
echo "PASS: jump 67, sneak 64, plain floor stayed, lone elevator stayed, blocked target skipped to 70"
echo "EXIT:0"
