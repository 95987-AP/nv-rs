# Nonblocking notification cards

User request, 2026-10-09: startup notices such as Classic Pack should appear
as subtle cards on the right and disappear after five seconds without a
click. This is an explicit presentation deviation from retail.

## Implementation

`viewer/src/game_menus/notifications.rs` owns presentation only. It fills
the installed game's `menus/message_menu.xml` using `MessageMenu`, hides
the button list, caches the resulting drawings, and reuses one tile tree.
The text, title, icon, fonts and frame come from the game. Card scale (0.75),
right-side width (30%), background opacity cap (160/255), spacing and
five-real-second lifetime are chosen for this request, not traced values.

Script boxes become cards only if their only visible button is OK (including
the localized `sOk`). Its original button index is acknowledged immediately
so `GetButtonPressed` scripts can continue. Engine `Popup` requests have
no script answer and also become cards. Questions and menus' own callbacks
continue through the original modal path. A card never enters `Screen.open`,
the interface's menu list or fade list, changes player readiness, or enables
the cursor. Up to three cards are shown; overflow waits for room and gets
five seconds of visible time. Window resizing preserves the card queue.

Original provenance: FalloutNV.exe 1.4.0.525 `005b4630` (script messages),
`00703f10` (engine popup), `007a8e60`/`007a92e0` (message queue/fill),
`00703fa0` (button result); existing translations documented in
`crates/ui/src/menus/message.rs` and `docs/OPENING.md`.

## Handoff

Branch: `codex/subtle-notifications`. Owned files: this page,
`docs/MILESTONES.md`, `viewer/src/game_menus/{message,mod,notifications}.rs`.
An existing staged `viewer/src/main.rs` FPS change is outside this batch.

Verified: root workspace tests, clippy, formatting and release build; all
171 viewer tests, clippy with warnings denied, formatting and release build.
Final spacing capture verified (`%TEMP%/nv-notification-final.png`, exit 0).

Live reference-cell runs (`GSDocMitchellHouse`, Vulkan, installed Data):
`--walk --background --run "ShowMessage PreorderMessageInventoryClassic"`
with `--screenshot` and waits of 1, 7 and 12 seconds. The startup generated
five pack notices (including the requested extra Classic Pack message).
Captures show right-side cards without OK buttons and the HUD/crosshair
still up; world AI continues in the log. Overflow appears after the first
three expire; the 12-second capture shows the stack cleared without input.
The capture exposed a top status-line overlap, so cards now begin 70 menu
units down; the final capture confirms that separation. Local evidence:
`%TEMP%/nv-notification-{visible,expired,cleared,final}.png`
and corresponding `.log` files (game assets/screenshots are not committed).
No original-game comparison is claimed: timing and placement deliberately
differ. The local checks include the separately staged FPS-counter change;
that file is not included in this branch's notification commit.

All check and viewer processes from this batch have exited. No unfinished
local checks. Next action: review the notification-card pull request.

## Local launcher follow-up (2026-10-09)

The user still saw centered boxes through `Launch-Playtest.cmd`: it calls
an extracted 2026-10-03.2 package's `Play.cmd`, which uses its old
`app/nv-viewer.exe` (last written 2026-10-07), not the rebuilt viewer.

The two local ignored launchers now pass/use `NV_RS_VIEWER`, pointing to
`viewer/target/release/nv-viewer.exe`. The root launcher checks that build
exists and gives the rebuild command if missing. Its selection menu and
the package's `userdata` directory are preserved. Direct package launching
without the override still uses the original packaged binary. The root
launcher labels the selected executable as a local development build rather
than showing the old official build metadata. No package binary or game
installation files were replaced.

Backups of both launchers and patches of the existing staged/unstaged work
are under `%TEMP%/nv-launcher-backup-20261009-212322`. Existing user edits
in `AGENTS.md`, `docs/MILESTONES.md` and the staged FPS counter were preserved.

Verified by running `Launch-Playtest.cmd` through its menu, selecting 1 and
inspecting the resulting process: `viewer/target/release/nv-viewer.exe`,
`GSDocMitchellHouse --official`. This uses the release already checked above,
including both registered UI customizations. The test launch is currently
open (viewer PID 20628, terminal session 29552); no additional Rust changes
or unchecked rebuilds. Next action: launch normally through the root launcher.
