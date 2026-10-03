# Opening Doc look IK trace

Research status, 2026-10-03. FNV executable addresses below refer to the
existing `nv-re/decomp/codex-m1` Ghidra project. No runtime behavior has been
implemented from this trace.

## Confirmed native path

Doc's actor update `008a3100` chooses a tracked actor, obtains its anchor from
the target actor's virtual `+0x194` method, and calls `008a3b70` on the actor's
controller stored at actor `+0xac`. `008a3b70` writes the supplied xyz into
controller `+0xd0` (fourth component zero), then invokes `00c75580` to enable
the look system. The controller is created by `0087e130` through constructor
`00c7f060`; its initialization path includes a specific LookIK initialization
failure diagnostic.

The visual consumer is verified, including register/stack roles:
`00c7d630` has the controller in ECX and passes `this+0x10` on the stack to
`00c7aa60` (`dis_00c7d630.txt`, `00c7d744-00c7d747`). The callee keeps ECX as
the controller and calls `00c78160` with that same `this`. `00c78160` reads
controller `+0xd0`, alongside its matrix at `+0x10`, and calls `00c7f840`.
That helper subtracts matrix translation from target xyz, applies the inverse
controller rotation to get a local direction, and normalizes it. Thus the
target-to-local-direction transform is established and this is the actor's
LookIK path, not a generic look-at inference.

`00c78160` records whether the local target is below in controller `+0x1a4`.
The below-target branch is conditional on state byte `+0xb2`; it uses signed
clamp values `+5` and `-5` before mapping through selected skeleton
transforms. It also limits movement of the smoothed point to 5 native units
per update (`DAT_011b05a8`, `00c78598-00c785f0`). The precise coordinate
meaning and units of those clamps are not yet established.

`00c7aa60` calls `00c78610` with `param_2=1` for the active result, then
passes a second `param_2=0` call using a temporary matrix. `00c78610` reads the
same target at controller `+0xd0`, selected skeleton index `+0x144`, and
per-mode index at `+0xf4`/`+0xf6`. Its body builds and composes quaternion
transforms and writes node transforms, but the Ghidra output is too large and
ambiguous to claim the final rotation formula or limits without tracing the
exact output writes and helpers.

## Unresolved, exact next trace

The controller constructor initializes short `+0x144` to `-1`. The setter is
now found in `00c79340`: it writes the result of `00cdd390` to
`controller + mode * 0x50 + 0xf4`; mode1 therefore writes `+0x144`.
`00c7de60` calls it with mode1. Its sole caller is `0087e575`, passing the
string returned through `0063d040 -> 00559450` from the body-part iterator
at local `-0x290`. **Assembly confirms the name is BPNI**: `0063d04a`
adds0x14 before the string getter; Ghidra's C output omitted that adjustment.
The loader `005e427c` checks BPNI and `005e42b0` writes its string at+0x14.
The iterator (`005e5320`) scans 15 BPTD part slots+0x34, selecting flag0x02;
`0087e940` selects flag0x20 for LookIK. DefaultBodyPartData0000001D's Head
entry has flags0x3b and BPNI `Bip01 Head`. Thus mode1's selected node is
data-driven, and is Bip01 Head for the standard human body data. BPNN is
different (Bip01 Neck1, stored at+4); do not confuse the two. The temporary
Ghidra project lock cleared. Evidence: look-node-getter.txt,
look-body-strings.txt and look-body-data.txt in nv-re/work/codex-m1.

For output rotation, continue `00c78610` from its `param_2=1` call in
`00c7aa60`: label each transform source (`+0x110..+0x11c`, per-node data,
target local direction), follow the final writes to the selected node's
rotation, and decompile only helpers on that dataflow. Separately trace
`00c755e0` if it contributes the angle or easing value. Confirm whether the
second (`param_2=0`) call is just a temporary/cache update. These are the
minimum gaps before a runtime implementation can reproduce native behavior.

The opening's `SayTo Player` is a scripted talk event, not a dialogue package;
it is an explicit reason for Doc to have the player as a target, but the
viewer currently has no native actor look-target/effect wiring. Random
`SitChairRelax` idle variation is separate from the absent gaze behavior.
`viewer/src/faces.rs` documents that head-turn and eye tracking are absent;
the seated body-turn guard in `viewer/src/ai.rs` must remain intact.

## Evidence locations

Decompilation and disassembly: `%USERPROFILE%\\nv-re\\decomp\\codex-m1`
(`008a3100`, `008a3b70`, `00c7f060`, `00c7d630`, `00c7aa60`, `00c78160`,
`00c7f840`, `00c78610`). Opening route and package evidence:
`docs/OPENING.md` and `%USERPROFILE%\\nv-re\\work\\codex-m1\\animation-repro.log`.
