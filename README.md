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
| Horai | golden pickaxe | Finds ore and digs to it. Evolves into Diamond, then Netherite Horai |
| Ooedo | TNT | Walks up to hostile mobs and explodes; no explosion hurts it |
| Germany | golden axe | Finds trees and chops them down |
| London | eye of ender | Reports the spawners above and below it |
| Russia | chest | 36 slots: picks up what is dropped around it, lights dark places with torches; bound to another doll with a lead, follows it. Evolves into Moscow |
| Moscow | (from Russia) | Follows the doll it is bound to and takes what it gathers |
| Marisa | fire charge | Shoots homing stars at hostile mobs |
| Reimu | paper | Shoots rings of homing amulets |
| Aya | feather | Takes photos (real screenshots, from her eyes, with no HUD); punch her with an empty hand for one of you |
| Wakasagi | tropical fish | Fishes; give her a fishing rod and she fishes with it and its enchantments |
| Goliath | diamond sword, iron blocks | Nearly three blocks tall: fights, knocks back, wears the armour in its first slots. Evolves into Netherite Goliath |
| Yuuka | sunflower | Tills, plants and harvests, replanting |

Right-click a doll empty-handed to switch modes (Standby, Patrol, Follow, Rideon); with a Doll Core
to turn its AI on or off; with a chest to open its inventory. Sneak and right-click it empty-handed
to pick it up: like a shulker box, its item keeps its inventory, love and experience. A doll on
your head stays out of your view in first person and keeps its reports to itself.

## Love

Dolls eat food from their inventory to heal, and at work (on patrol, following or on your head) a
piece every five minutes. They pick up experience orbs, which you take back with a glass bottle.
Eating earns love, but not healing what you did to them yourself: hit your own doll, and the food it
eats until it is whole again earns nothing. Point and power items (dropped by
[Touhou Mobs](https://github.com/mxpbxb/touhou-mobs), from
[Touhou Items](https://github.com/mxpbxb/touhou-items)) earn love too: right-click to give the stack,
sneak-click to give one.
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
- Port by mxpbxb.

Touhou Project is by ZUN; this is a non-commercial fan work. No Minecraft code or assets are included.
