# The HUD's corner messages and their pictures

The messages in the HUD's top-left corner (`hud_main_menu.xml`'s message
bracket) each have a picture beside them. Read from FalloutNV.exe
1.4.0.525:

- `QueueUIMessage` (`007052f0`, the HUD's `00775380`) takes the text, a
  type, a picture, a sound, how many seconds and whether it goes first.
  Without a picture the type picks a Vault Boy: 0 neutral, 1 very happy,
  2 sad, 3 in pain.
- A script's `ShowMessage` of a message that isn't a box (`005b4630`)
  passes the message's own picture: its `INAM`, a menu icon record
  (`MICN`) whose `ICON` is the picture; none, the neutral Vault Boy.
- The game's own messages pass a picture per call. Each of the 80 callers
  of `007052f0` was read for the text setting it shows and the picture it
  pushes: `world::message_icon::for_setting` has the ones with one picture
  (sad for the sleep, wait, fast travel, equip and save refusals; in pain
  for rising radiation and hardcore needs, very happy when they fall; the
  padlock for locks needing a key or more skill, the key for opening with
  one; surprised for the casinos' refusals, "out of lockpicks", a broken
  lock, an essential person down, being over-encumbered; the gift box for
  "added"; the map for a new map marker, the radio tower for a station).
  `sHackIneligible` has two (angry `00501310`, sad `005015f0`) and isn't
  listed.
- Reputation changes read their picture from a setting
  (`sRepPositiveGainIcon`…, `00615730`…), karma from `sKarma…Image`
  (`0094fd30`).
- The casino games: refusals and "out of chips" surprised, chips lost sad,
  the ban very happy (`00734db0`, `007bd2a0`, `007c2c70`), chips won the
  "added" notice's gift box (`004821a0`, `0101c140`).

## Here

`world::scripting::Event::Message` carries the picture (`icon`; `None` the
neutral Vault Boy); `world::message_icon` has the pictures, the table and
`of_message`. The viewer's `hud::HudMessages` queue holds `HudMessage`s
(text and picture) for `ui::hud::Hud::queue_message`. Given pictures: a
script's `ShowMessage`, the casinos' messages, the lockpicking menu's,
locked doors, broken equipment (`sCantEquipBrokenItem`), reputation and
karma, hardcore needs and radiation, the sleep/wait refusals of
`ShowSleepWaitMenu`. Other messages still have the neutral Vault Boy
(dialogue and pickpocket refusals, a companion over-encumbered, weapon
condition and breaking, experience, challenges, discovered places).
