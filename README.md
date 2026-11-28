# Alice Dolls Restringed

Alice's dolls for [Horizon](https://playhorizon.dev): a port of **Alice's Dolls** (1.7.10-kog3, by
iwa_yukinoshita), used with permission, with its extra dolls and new ones, restrung with love
levels, evolutions and new jobs.

## Two parts

Install both from the release, then press Build in the launcher:

- **`alice_dolls_native`** (`.hmod`, native): the code. A native mod: it runs with your account's
  permissions, so the launcher asks before building it.
- **`alice_dolls`** (`.hmod`): the doll items, recipes, names, sounds and textures.

## Dolls

Each recipe is the item in the top left, wool (Goliath: iron blocks), and a Doll Core in the middle. Patchouli and Alice also take three diamonds, in the other corners. Joon is a Doll Core ringed with gold and emerald blocks, gold in the corners.

| Doll | Crafted with | What it does |
|---|---|---|
| Shanghai | golden sword | Fights hostile mobs, shears sheep, reports the mobs around it. Evolves into Goliath |
| Horai | golden pickaxe | Finds ore (ancient debris too) and digs to it. Evolves into Diamond, then Netherite Horai, which earns no love for digging, only 5 for each quarry finished, a whole level for each giga quarry, and 10 for each diamond she digs out of either, and waits four seconds before heading off to its quarry so you can give it another order. Sneak and right-click any Horai with a golden pickaxe for the giga quarry: a 20x20 pit round where she stands, dug layer by layer down to bedrock. When it is done, or you give her another order, she leaves a line of ladders down the pit's wall from the corner where she began |
| Ooedo | TNT | Walks up to hostile mobs and explodes; no explosion hurts it |
| Germany | golden axe | Finds trees and chops them down, half as quick again as an axe, stump and all; with a sapling of that wood in her inventory she plants it where the tree stood |
| London | eye of ender | Reports the spawners above and below it |
| Russia | chest | 36 slots: picks up what is dropped around it, lights dark places with torches; bound to another doll with a lead, follows it. Evolves into Moscow |
| Moscow | (from Russia) | 54 slots of 128, one slot and two to a stack more for each level after it evolves (to 64 slots). Follows the doll it is bound to and takes what it gathers |
| Marisa | fire charge | Shoots homing stars at hostile mobs |
| Reimu | paper | Shoots rings of homing amulets |
| Aya | feather | Takes photos (real screenshots, from her eyes, with no HUD); punch her with an empty hand for one of you. Riding on your head she gives you Speed II, and two levels more for each other Aya riding you |
| Wakasagi | tropical fish | Fishes; give her a fishing rod and she fishes with it and its enchantments. From love level 3 she fishes with at least Lure III and Luck of the Sea III of her own |
| Goliath | diamond sword, iron blocks | Nearly three blocks tall: fights, knocks back, wears the armour in its first slots. Evolves into Netherite Goliath |
| Yuuka | sunflower | Tills, plants and harvests, replanting; cuts cactus, sugar cane and bamboo down to their bottom block, which grows again; picks pumpkins whole, as with Silk Touch; farmland trampled near her in the last five minutes she tills and plants again. |
| Yukari | ender pearl | Follows you, her one mode, and leaves mobs and dropped items alone. She is a quarter taller than the other dolls. With [Yukari Gaps](https://github.com/mxpbxb/yukari-gaps) she is your portable gap: right-click her empty-handed for the travel screen, and she comes through after you. She is no gap to travel to |
| Patchouli | book | A sniper: one slow, heavy orb every four seconds at hostile mobs up to 64 blocks off. She walks at two thirds of a doll's pace, and riding on your head she slows you. |
| Alice | string | Right-click her with an empty hand for her panel: all your other dolls, wherever they are, with what each is doing, its love level and how far off it is. Tick dolls, then set them to Standby, Patrol, Follow or Rideon, turn their work AI on or off, ring them back as the bell does, open one's inventory from afar, send their items to another doll, or put them in your inventory as their items (any that don't fit come to you instead). She patrols from the moment she is placed |
| Joon | gold and emerald blocks | A trader with a large chest of stock (54 slots) and no armour or baubles. She follows from the moment she is placed. Right-click her for her screen, with two tabs. **Trades**: up to twelve, villager style, anything for anything: each takes up to four stacks and gives up to four, all at once (either side may be left empty, for a gift). Pick a trade from the list and one of its slots, then click one of your items to put it in (right-click puts in one), or switch to **All items** and type to search every item in the game (`water_bucket`, or a whole id such as `minecraft:water_bucket`; Enter takes the first found); set the count with - and + or the mouse wheel. **Her stock** opens her inventory: fill it with what she pays out. **Send**: the players in your dimension; click one to send her. She appears at their side and follows them for 200 seconds, and nothing can hurt her meanwhile. They right-click her for her trade offer and click Trade (shift- or right-click to trade as many times as they can pay and she has stock for); what they pay goes into her inventory, and a trade she is out of is crossed out. They can send her home at any time. She can't visit the same player again for 300 seconds; you can send her to yourself whenever you like, to try her trades |

Fighting dolls go first for whatever is attacking them, and go for iron golems that turn on you. A Goliath under half its health hits twice
as hard, and a hostile mob it strikes turns on it rather than on its owner. When a doll is down to
under 30% of its health, every doll within 16 blocks fights whoever hurt it (those that don't fight
otherwise too), and if its owner is not within 24 blocks the nearest of the owner's dolls within 8
picks it up and carries it as its item, with all it carries. A doll emptying itself into the chest
keeps its first stack of food.

Armour in a doll's armour slots (its last four) counts against every blow, as a player's does.

Horai dolls mine coal and iron as with Fortune III (the rest with Silk Touch).

A doll belongs to the player who placed it: it follows, rides and takes orders from them alone, and
only they can pick it up, even with a Doll Core.

Right-click a doll empty-handed to switch modes (Standby, Patrol, Follow, Rideon; on Standby it stays on the block where it was set to Standby, and is put back on it if knocked off); with a Doll Core
to turn its AI on or off; with a chest to open its inventory. Ring the doll bell to pick which of your
dolls come back to you (tick them, then ring from the screen); sneak and ring it to bring them all. Sneak and right-click it empty-handed
to pick it up: like a shulker box, its item keeps its inventory, love and experience. A doll on
your head stays out of your view in first person and keeps its reports to itself, and its baubles
protect you too (the fire bauble from lava and fire). When you respawn, the dolls that were following
you or on your head come back to you.

Each player can have one Horai out at a time (in any of its three forms), and two each of
Patchouli, Reimu, Marisa, Yuuka and Wakasagi.

## Love

Dolls eat food from their inventory to heal (rotten flesh too), and at work (on patrol, following or on your head) a
piece every five minutes. They go and get experience orbs within 16 blocks when they have no dropped item to fetch, and you take the experience back with a glass bottle.
Working earns love too: every 20 blocks mined, trees chopped, crops tended, fish caught or blows
landed. Eating earns love, but not healing what you did to them yourself: hit your own doll, and the food it
eats until it is whole again earns nothing. Point and power items (dropped by
[Touhou Mobs](https://github.com/mxpbxb/touhou-mobs), from
[Touhou Items](https://github.com/mxpbxb/touhou-items)) earn love too: right-click to give the stack,
sneak-click to give one. A Max Doll Point is worth four Point items. Points a doll carries in its
inventory it takes for itself, except Russia and Moscow, which keep them for you. Sneak-click Marisa, Reimu,
Patchouli, Yukari, Yuuka or Wakasagi with a bomb (Arcadian Dream's or Touhou Items') and she takes it
and drops a limited spell card of her character's, from Touhou Items: Stardust Reverie, Fantasy
Seal, Agni Shine, Mesh of Light and Darkness, Flower Wind Moon and Tail Fin Slap. It casts 16
times. Dolls never drown. Russia and Moscow move twice as fast as other dolls. They go for dropped items within 20 blocks of the doll they follow, and once a second pull what your other dolls within 10 blocks carry into their own inventory: all but what a doll keeps when it empties itself into a chest. They earn 3 love for each stack they pick up, and 3
for each quarter hour they follow the doll they are bound to.
Each level takes three quarters more than the last, and gives 3 more attack and 5 more health, each
then 10% more, and makes the working dolls faster and stronger: mining, blasting, farming and fishing.
Every three levels, a doll with a next form evolves. A doll's name tag shows its level, how far it
is to the next, and its health.

A doll that kills a zombie, skeleton, creeper, wither skeleton, piglin or the ender dragon gets its
head, and picks it up.

## Building

`content/` is the content mod; build it with `cargo horizon-mod build`. It also declares the doll
mob, `alice_dolls:doll`: a mob of its own (a small ground mob's body, the doll's size), not a
chicken, so nothing that hunts or herds chickens touches a doll.

`native-sdk/` is the native mod for today's launcher (SDK packages, from 0.43.0): `src/` keeps each
doll's owner, mode, inventory and love and draws its model, and `src/mixins/` (`goals.rs` and the
hooks) gives the dolls their jobs. Pack it with `cargo horizon-mod build` against the Horizon SDK
it pins; the launcher builds it (`native-build`). `native/` is the old `.hnmod` form, up to 0.42.7.

## Credits

- **Alice's Dolls** by iwa_yukinoshita: the dolls, their designs, models and art, used with permission.
- **Touhou Little Maid** by TartaricAcid and others: the doll bell, doll anchor and bauble art, under
  [CC BY-NC-SA 4.0](https://creativecommons.org/licenses/by-nc-sa/4.0/); its love levelling
  inspired this port's.
- The Yukari, Patchouli and Alice dolls' models and art are mxpbxb's.
- Port by mxpbxb.

Touhou Project is by ZUN; this is a non-commercial fan work. No Minecraft code or assets are included.
