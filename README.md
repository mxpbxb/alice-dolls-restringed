# Alice Dolls Restringed

Alice's dolls for [Horizon](https://playhorizon.dev): a port of **Alice's Dolls** (1.7.10-kog3, by
iwa_yukinoshita), used with permission, with its extra dolls and new ones, restrung with love
levels, evolutions and new jobs.

## Two parts

Install both from the release, then press Build in the launcher:

- **`alice_dolls_native`** (`.hnmod`): the code. A native mod: it runs with your account's
  permissions, so the launcher asks before building it.
- **`alice_dolls`** (`.hmod`): the doll items, recipes, names, sounds and textures.

## Dolls

Each recipe is the item in the top left, wool (Goliath: iron blocks), and a Doll Core in the middle.

| Doll | Crafted with | What it does |
|---|---|---|
| Shanghai | golden sword | Fights hostile mobs, shears sheep, reports the mobs around it. Evolves into Goliath |
| Horai | golden pickaxe | Finds ore (ancient debris too) and digs to it. Evolves into Diamond, then Netherite Horai, which earns no love for digging, only 5 for each quarry finished, and waits four seconds before heading off to its quarry so you can give it another order. Sneak and right-click any Horai with a golden pickaxe for the giga quarry: a 20x20 pit round where she stands, dug layer by layer down to bedrock |
| Ooedo | TNT | Walks up to hostile mobs and explodes; no explosion hurts it |
| Germany | golden axe | Finds trees and chops them down |
| London | eye of ender | Reports the spawners above and below it |
| Russia | chest | 36 slots: picks up what is dropped around it, lights dark places with torches; bound to another doll with a lead, follows it. Evolves into Moscow |
| Moscow | (from Russia) | Follows the doll it is bound to and takes what it gathers |
| Marisa | fire charge | Shoots homing stars at hostile mobs |
| Reimu | paper | Shoots rings of homing amulets |
| Aya | feather | Takes photos (real screenshots, from her eyes, with no HUD); punch her with an empty hand for one of you |
| Wakasagi | tropical fish | Fishes; give her a fishing rod and she fishes with it and its enchantments. From love level 3 she fishes with at least Lure III and Luck of the Sea III of her own |
| Goliath | diamond sword, iron blocks | Nearly three blocks tall: fights, knocks back, wears the armour in its first slots. Evolves into Netherite Goliath |
| Yuuka | sunflower | Tills, plants and harvests, replanting; farmland trampled near her in the last five minutes she tills and plants again. |
| Yukari | ender pearl | Follows you, her one mode, and fights hostile mobs. With [Yukari Gaps](https://github.com/mxpbxb/yukari-gaps) she is your portable gap: right-click her empty-handed for the travel screen, and she comes through after you. She is no gap to travel to |
| Patchouli | book | A sniper: one slow, heavy orb every four seconds at hostile mobs up to 64 blocks off. |
| Alice | string | Nothing yet |

Fighting dolls go first for whatever is attacking them. A Goliath under half its health hits twice
as hard, and a hostile mob it strikes turns on it rather than on its owner. When a doll is down to
under 30% of its health, every doll within 16 blocks fights whoever hurt it (those that don't fight
otherwise too), and if its owner is not within 24 blocks the nearest of the owner's dolls within 8
picks it up and carries it as its item, with all it carries. A doll emptying itself into the chest
keeps its first stack of food.

Horai dolls mine coal and iron as with Fortune III (the rest with Silk Touch).

A doll belongs to the player who placed it: it follows, rides and takes orders from them alone, and
only they can pick it up, even with a Doll Core.

Right-click a doll empty-handed to switch modes (Standby, Patrol, Follow, Rideon); with a Doll Core
to turn its AI on or off; with a chest to open its inventory. Ring the doll bell to pick which of your
dolls come back to you (tick them, then ring from the screen); sneak and ring it to bring them all. Sneak and right-click it empty-handed
to pick it up: like a shulker box, its item keeps its inventory, love and experience. A doll on
your head stays out of your view in first person and keeps its reports to itself, and its baubles
protect you too (the fire bauble from lava and fire). When you respawn, the dolls that were following
you or on your head come back to you.

## Love

Dolls eat food from their inventory to heal, and at work (on patrol, following or on your head) a
piece every five minutes. They pick up experience orbs, which you take back with a glass bottle.
Working earns love too: every 20 blocks mined, trees chopped, crops tended, fish caught or blows
landed. Eating earns love, but not healing what you did to them yourself: hit your own doll, and the food it
eats until it is whole again earns nothing. Point and power items (dropped by
[Touhou Mobs](https://github.com/mxpbxb/touhou-mobs), from
[Touhou Items](https://github.com/mxpbxb/touhou-items)) earn love too: right-click to give the stack,
sneak-click to give one. A Max Doll Point is worth four Point items. Points a doll carries in its
inventory it takes for itself. Russia and Moscow move twice as fast as other dolls. They earn 3 love for each stack they pick up, and 3
for each quarter hour they follow the doll they are bound to.
Each level takes three quarters more than the last, and gives 3 more attack and 5 more health, each
then 10% more, and makes the working dolls faster and stronger: mining, blasting, farming and fishing.
Every three levels, a doll with a next form evolves. A doll's name tag shows its level and how far it
is to the next.

A doll that kills a zombie, skeleton, creeper, wither skeleton, piglin or the ender dragon gets its
head, and picks it up.

## Building

`content/` is the content mod; build it with `cargo horizon-mod build`. `native/` is the native mod,
built by the Horizon launcher (`native-build`), pinned to one Horizon revision: `api/` keeps each
doll's owner, mode, inventory and love and draws its model, and the mixin files (`goals.rs` and the
hooks) give the dolls their jobs.

## Credits

- **Alice's Dolls** by iwa_yukinoshita: the dolls, their designs, models and art, used with permission.
- **Touhou Little Maid** by TartaricAcid and others: the doll bell, doll anchor and bauble art, under
  [CC BY-NC-SA 4.0](https://creativecommons.org/licenses/by-nc-sa/4.0/); its love levelling
  inspired this port's.
- The Yukari, Patchouli and Alice dolls' models and art are mxpbxb's.
- Port by mxpbxb.

Touhou Project is by ZUN; this is a non-commercial fan work. No Minecraft code or assets are included.
