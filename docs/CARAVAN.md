# Caravan

Caravan is New Vegas's card game (`CaravanMenu`, class 1083,
`menus\caravan_menu.xml`), played against people whose dialogue calls
`ShowCaravanMenu <deck> <difficulty> <bet share>` (27 calls in 18 scripts:
Ringo, the Mojave Outpost's soldiers, the Tops' high rollers…). Until
2026-10-06 nv-rs had none of it.

The PC exe inlines most of the game into the menu's update (`00741500`,
26 KB). The Xbox 360 build's PDB names its parts, and its code
(decompiled from `Fallout_Release_MemDebug.xex`, section 4 at
`0x82250000`) was matched to the PC's piece by piece; both use the same
member offsets (`trackInfo` at +0xDE4, the tracks at +0xB44).

## Read from the game

| What | Xbox 360 (PDB) | PC |
| --- | --- | --- |
| A card's suit and value | `TESCaravanCard::Load` | |
| A deck's cards | `TESCaravanDeck::Load` | |
| `AddCardToPlayer` | `PlayerCharacter::AddCaravanCard` | `005cf3d0` → `00969bc0` |
| `ShowCaravanMenu` | `CaravanMenu::Create` | `005cf250` → `00741060` |
| Whether a card may go on a row | `IsValidCardPlacement` | `0074f6c0` |
| A track's value, way and suit | `UpdateTrackValue` | `0074ee70` |
| Under / sold / over | `CalculateWinningState` | |
| The end | `IsGameOver` | |
| The deal | `PrepareGameMenu` | |
| The player's moves | `DoGamepad` | `00747d30` |
| The turns, jacks, jokers, the opponent's moves | `DoIdle` | `00741500` |
| Which controls work | `UpdateCaravanFlags` | |

- Cards (`CCRD`): the first `INTV` the suit (1 hearts, 2 spades, 3
  diamonds, 4 clubs, 5 blank), the second the value (1 ace, 2–10, 12 jack,
  13 queen, 14 king, 15 joker); `TX00`/`TX01` the face and back. Decks
  (`CDCK`): `CARD`s.
- The player's cards are two lists on the player (+0x624 outside the
  deck, +0x628 in it), not things carried: a card's script
  (`CardAddToPlayerScript`) calls `AddCardToPlayer` when the player picks it
  up and `RemoveMe`. A card already in either list isn't added again. The
  player plays with at least 30 cards (`UpdateCaravanFlags`).
- The table: six tracks of up to seven rows; tracks 0–2 the player's, 3–5
  the opponent's, track t against 5 − t. A row is a number card and the
  face cards played on it.
- The deal (`PrepareGameMenu`): the player's deck is their in-deck cards,
  the opponent's the `ShowCaravanMenu` deck; eight cards each, the
  player's then the opponent's, each picked at random from what's left; then
  each side's next card is picked at random. Decks are never shuffled.
- Starting the caravans (flag `CAF_TRACK_SETUP` 0x800): number cards only;
  the player's go on their first empty track (0, 1, 2), the opponent's on
  its last empty one (5, 4, 3), row 0; nothing is drawn. Throwing a card
  away then draws one and keeps the turn. It ends when tracks 2 and 3 both
  have a card.
- Placing (`IsValidCardPlacement`): a number card only on a new row of the
  side's own track, never the same value as the number card before it, and
  only the track's way (up after a higher card, down after a lower) unless
  it's the track's suit. A face card on a row with its number card and at
  most two others, on any track; a queen only on a track's last row.
- A track's value (`UpdateTrackValue`): each row's number card, doubled for
  each king on it. Its way: up when its last number card is higher than the
  one before, else down (none with one card). Its suit: the last number
  card's, each queen on the last row changing it to hers and turning the way
  round.
- After placing (`DoGamepad`, state 21): the card leaves the hand and,
  past the start, the side draws its next card to the end of its hand (and
  the next is picked at random). Jacks (state 17, 18) take away their row
  (the number card, everything on it, the jack), the rows after it moving
  up. Jokers (states 13, 19) stay and take away every other row whose number
  card has the same value as theirs, or for a joker on an ace the same
  suit, on any track.
- Throwing away a card draws one; throwing away a whole track empties it
  (state 15).
- Turns (state 21, the byte at +0xE82): the opponent moves first, then
  each in turn; a card thrown away while starting the caravans keeps the
  turn.
- The opponent (`ProcessAI`, `0074fdc0`; `CaravanAIPackage`): every move it
  could make becomes a package (the value it would bring a track to, how
  that stands, the action, track, row and hand card, a priority); sorted by
  priority with the exe's own `qsort` (Visual Studio 2008's, `00ec6f20`, so
  equal priorities fall as in the game), the first is played. For each of
  its tracks: under 21, number cards and kings that don't pass 26
  (priority the new value; kings +1, −5 at difficulty 2); sold but beaten
  by the player's facing track, number cards (priority 100 − the margin
  over it, 50 for a tie, else 25 − what's short of 26), jacks on rows
  whose loss would put the player below it (75 + or 100 − the margin) and,
  above difficulty 0, jokers (an estimate of what each track loses; at
  difficulty 3 a score over all three pairs); sold and ahead, number cards
  and kings that stay sold; over 26, jacks that bring it back (100 −
  (26 − value + the row)). On the player's tracks: jacks on sold (51 −
  what's left) or short (26 − what's left) tracks, kings that push one
  over (its value − 1). With nothing to play it throws away its highest
  card (priority: its place in the hand) or one of its unsold tracks
  (value − 26; × 2 above difficulty 0; − 5 a jack in hand at 3). Starting
  its caravans: easy plays its last number card unless it has six face
  cards (then throws its last face card away); 1–2 keep 6–10s or 1–3s and
  throw a face card away with too few (a joker first; a jack gives way to
  anything, a king to anything but a jack); 3 plays the card nearest 6.
- The game's oddities, kept: a jack meant for a player's row is checked
  and played on the opponent's own track at that row number; the joker
  estimate counts aces by value on an ace and otherwise sums the rows whose
  suit number equals the card's value; a king counts once under 21 and
  twice when sold; the queen move on the player's tracks can never happen
  (it asks for an empty row inside a loop over non-empty ones).
- The end (`IsGameOver`, before every turn): a caravan is sold at 21–26
  and wins its pair if the other isn't sold or is lower; all three pairs
  decided, the side with more wins. Otherwise a side with no cards in deck
  or hand left loses (the player checked first).

## Here

`world::caravan`: `card`, `deck`, `Collection` (`GameState::caravan`,
saved as `caravancard` / `caravanrecord` lines), `add_card_to_player`
(the `AddCardToPlayer` command), `Game` (`new`, `is_valid_placement`,
`update_track_value`, `is_game_over`, `play` and `resolve`, `discard`,
`discard_track`, `setup_track`, `setup_finished`, `npc_turn`,
`player_play`), `winning_state`; `world::caravan::ai` (`process_ai`,
`Package`, `crt_qsort`). Checks: `world::caravan::tests`,
`world::caravan::ai::tests` (39 whole games each end),
`crates/world/tests/caravan.rs`.

## Not done yet

- The ante and the bet (`PrepareAnteMenu`, `ExchangeCurrency`,
  `UpdateBettingUI`), the results and the player's record
  (`PrepareResultsMenu`, misc stats "Caravan Games Won/Lost").
- The deck building screen and the menu itself (`caravan_menu.xml`, the 3D
  table `Meshes\Terminals\NV_Caravan\`), and `ShowCaravanMenu` opening it.
