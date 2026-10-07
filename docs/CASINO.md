# The casino games

Blackjack, roulette and the slot machines, as FalloutNV.exe 1.4.0.525 runs
them (the Xbox 360 prototype's PDB naming the parts; PC addresses below).
`world::casino` has what the three share and each game's rules and menu
states; the tiles, the 3D and the viewer follow Caravan's layout
([CARAVAN.md](CARAVAN.md)).

## The record (`CSNO`, `TESCasino`, loader `005048d0`)

- `FULL` the name the earnings line shows (Gomorrah's is "Gommorah" in
  FalloutNV.esm).
- Ten models: `MODL`s in order (the chip stacks for 1, 5, 10, 25, 100 and
  500, the roulette chip, the slot machine), then `MOD2` the slot machine
  again (the editor writes both), `MOD3` the blackjack table, `MOD4` the
  roulette table.
- Eleven textures: `ICON`s the reels' symbols (A to F, then the wild:
  Symbol1, 2, 3, 4, 6, 7, 5 in every record), `ICO2`s four card backs.
- `DATA` (`CASINO_DATA`, 0x38 bytes): blackjack's shuffle point (0 means
  0.25) and blackjack payout (0 means 1.5), the seven reel symbols' stops,
  the decks (0 means 1), `iMaxWinnings`, the chip (`CHIP`), the comps
  quest, the dealer's hole card flag.
- Chips (`CHIP`) have no `DATA`: worth nothing, weightless. The cashiers'
  dialogue swaps caps (and NCR money) for them.

## What the games share

- **The player's list** (`PlayerCharacter` +0x610, `CasinoData`): per
  casino the net chips won by every round of every game there (it can go
  below 0) and the level reached. Each game's `Create` adds the casino's
  line first (at the head), even when it then refuses. Saved.
- **Levels**: k (1 to 4) at `winnings ≥ k × 0.25 × iMaxWinnings`. After a
  round's payout (blackjack `00736b56`, roulette `007be412`, the slots after
  their coins `007c360d`) a level above the stored one is stored and the
  menu closes; closing then starts the casino's comps quest
  (`0060c9c0(quest, 1)`). The quests' scripts and the floor managers'
  dialogue hand out the comps (`GetCasinoWinningStage` conditions).
- **Script functions**: `GetCasinoWinningsLevel` (`005dec30`) and the
  condition `GetCasinoWinningStage` (`005a6170`) work the level out from the
  winnings (so it drops with losses), 0 where not played;
  `SetCasinoWinningsLevel` (`005ded40`) sets a level and its winnings;
  `SetCasinoCheatLevel` does nothing on PC.
- **Opening** (`ShowSlotMachineMenuParams`, `ShowBlackJackMenuParams`,
  `ShowRouletteMenuParams`: casino, least bet, most bet, and a fourth
  number the command calls "Max Winnings" that's really the least winnings
  to sit down; blackjack's handler drops it). `Create` refuses in turn,
  each as a corner message with `UIPopUpMessageGeneral`: the anti-cheat
  lock, banned (winnings at the limit), too few chips for the least bet,
  too little won.
- **The anti-cheat lock** (not saved): closing a casino menu stamps the
  time (`00969b20`); loading a game while stamped arms it (`00969ac0`);
  armed, every game is refused for `iAntiCheatDuration` (60) seconds after
  the stamp ("…as an anti-cheating measure.\nTime Remaining: N").
- **Chips**: the menu keeps its own count from the player's and settles
  once on closing: fewer, the difference is taken ("N <chip>(s) removed",
  sad Vault Boy); more, given (the usual "added" notice, or at the limit
  quietly with "N <chip>(s) added\nYou have been banned from gambling at
  this casino.").
- **Texts**: the earnings line `"%s %s%i"` (name, "Earnings: ", winnings;
  its padding to 23 characters only takes with a one-character number),
  round results `"%s %d %s%s"` ("You win 5 chip(s)", "You feel lucky. You
  win…").

## The slot machines (`SlotMachineMenu`, class 1080)

- **The strip** (`SetupActiveReels` `007c16b0`): 28 places filled round
  by round, each symbol with stops left putting itself (2k) and its blank
  (2k + 1); the stops must come to 14 or the menu closes with an error.
- **A spin** (`SpinReels` `007c56e0`): each reel `strip[rand(0, 2n − 1)]`
  (n the symbols with stops), so only the first round of the strip counts
  and its last blank never comes up: in the vanilla casinos 13 outcomes a
  reel. Everything is decided on the click.
- **Results** (`InterpretSpin` `007c5760`): three of a kind (6) pays the
  bet × 10, 20, 30, 40, 60, 100 for bell, BAR, 7, lemon, grapes, orange,
  × 10 for three cherries (the wild); two wilds (5) × 5; one (4) × 2;
  anything else loses the bet (pairs too). (The Xbox prototype and the
  first payout texture paid 80 and 160 for grapes and orange.)
- **Luck** (`ApplyLuck` `007c58f0`, Luck read once at the start): acts when
  `rand(0, 100) + |Luck − 5| × 10 > 100`. Below 5: wilds turn blank, three
  of a kind slips (A, B, C to blanks; D, E, F and wilds to three bells);
  the "unlucky" flag it sets is lost to its return of 0, so "You feel
  unlucky. You lose" never shows and losses are silent. Above 5: a loss
  spins again (luck rolled again), a pair whose odd reel is within Luck
  (squared distance in ids) becomes three, a wild's neighbours within Luck
  match.
- **States** (`DoIdle` `007c2c70`; seconds counted only while the menu is
  on top): the lever and reels (`GameSlotsPullLever`; the window's faces
  retextured as the lever ends, the rest as the reels stop), the result
  (one update: paid, the status line), the coins for a win
  (`GAMESlotsWinSmall`, `Med`, `Jackpot`; then the level check), the
  payout card in and out, idle (out of chips: "You must purchase chips…"
  and the menu closes; fewer chips than the bet sets it to 1).
- **Clicks** (`DoClick` `007c2460`; W, E, Q, F, S, R by the menu's own
  keys): Spin (with chips for the bet; counts "Slots Games Played"),
  Increase (+1 under 10, 5 under 40, 10 under 100, 25 under 500, else
  100; no more than the chips or the most bet; `GAMESlotsIncreaseBet01`
  to `03` in turn), Decrease (back by the same steps to their multiples;
  no less than the least bet; `GAMESlotsDecreaseBet`), Payout List, Bet
  Max, Exit. With the payout card in, any click takes it out.
- **The reels' faces** (`SwapFrontfacingTextures` `007c4d70`,
  `SwapBackfacingTextures` `007c40b0`): each reel's 14 two-sided meshes
  `Cylinder0{6 − reel}:{face}`, its payline face 2 × reel; the window's
  six faces show the result with its neighbours, the other eight skip a
  symbol and repeat one (only seen spinning).
- Here: `world::casino::slots` (`SlotMachine`, its `Effect`s).

## Here

- `world::casino`: the record, the player's list (saved as `casino`
  lines), levels, the script functions, `open_check` and the lock,
  `settle` (with its corner message), the texts; `Event::Casino` from the
  three commands.
- `ui::menus::slots` (the tiles), `cellview::slots` (the 3D).
- The viewer: `Event::Casino` runs `Create`'s checks where the script asks
  (`game_menus::casino::create`: a refusal is a corner message with
  `UIPopUpMessageGeneral`), then `menus::Menu::Casino` opens the game's
  menu (`game_menus::slots`); `casino_scene` draws the machine with the
  lockpicking menu's camera into the HUD's picture, under the menu's
  tiles (the HUD's camera blends over any menu's 3D:
  `game_menus::scene_open`). The anti-cheat lock (`CasinoLock`) runs on
  the real clock, stamped as a menu closes and armed by F9's load.
- Checks: `crates/world/tests/casino.rs`, the module tests, the viewer's
  `game_menus::casino` test.

## Trying it

```
nv-viewer <Data> VikkiAndVance --freeze-ai --run "player.additem VikkiVanceChip 100" --run "0016202A.Activate player 1"
```

(`0016202A` is one of Vikki & Vance's slot machines, `vVikkiVanceSlotScript`;
`--freeze-ai` keeps people from starting a conversation that would hold the
second line back.) W spins, E and Q change the bet, F shows the payout card,
S bets the most, R leaves: the chips are settled with a corner message.

## Not done

- Blackjack's and roulette's rules and screens.
- The corner messages' own Vault Boy icons (the viewer's HUD messages
  have one icon for all).
