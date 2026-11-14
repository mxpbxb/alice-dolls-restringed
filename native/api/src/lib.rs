//! Alice's Dolls for the eltanschauung/horizon fork: a port of the 1.7.10 mod by iwa_yukinoshita.
//!
//! Alice's dolls: Shanghai fights and steals, Horai finds and mines ore, Ooedo blows itself up next
//! to monsters, Germany chops trees, London finds spawners, Russia gathers what is dropped and
//! lights the dark with torches. The extras: Marisa and Reimu shoot danmaku, Aya takes photos,
//! Goliath fights, Yuuka farms; Right-click one empty-handed to go Standby, Patrol, Follow, Rideon
//! and back to Standby; with a Doll Core to switch its AI on and off, or to take over someone
//! else's doll; with a book to change how much it says; with a chest to open its inventory. Sneak
//! and right-click it empty-handed, hit it with a Doll Core, or kill it, to get the item back.
//!
//! The fork has no living mod entity types, so a doll is a silent chicken with a custom name: this
//! mod draws it, and `goals.rs` drives it. A chicken is close to the doll's size and falls slowly,
//! as the doll did. This crate saves each doll's state and hands it to the goals each tick.

mod model;

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use horizon_client::mobs::alice_doll_goals as goals;
use horizon_client::mod_api::{
    CameraPoseEvent, HudMode, RenderNameTagEvent,
    AttachmentCodec, AttachmentError, AttachmentType, AttackEntityEvent, Compound,
    EntityTickListenedEvent, EntityTickPostEvent, Flow, Holder, Hooks, Host, InteractEntityEvent,
    ItemStack, LivingIncomingDamageEvent, LootDropsEvent, MenuClickedEvent, MenuClosedEvent, MenuInputClosedEvent,
    ScreenRenderPostEvent, ScreenMouseScrolledPreEvent, RightClickBlockEvent,
    MobSubmitEvent, NativeServerPlayer, PlaceBlockEvent, StackData, Tag, Text, TooltipEvent,
    UseItemEvent, UseItemOnEvent, WorldSubmitEvent, Direction, EquipmentSlot, LivingDeathEvent, NativeEffect, ClientTickPostEvent,
};

// The kind every doll is underneath.
const KIND: &str = "minecraft:chicken";
const DOLL_CORE: &str = "alice_dolls:doll_core";
/// Rung, it brings every one of its owner's dolls to them, on standby.
const DOLL_BELL: &str = "alice_dolls:doll_bell";
/// What a Moscow doll stacks everything to.
const MOSCOW_STACK: u32 = 128;
/// `DollBase.getSizeInventory`: one row.
const INVENTORY: usize = 9;

/// One of the dolls: the name that makes a chicken this doll, its item, its texture, what it holds
/// (`getHeldItem`, empty for nothing), how many slots its inventory has (`getSizeInventory`), and
/// how it is drawn.
#[derive(Debug)]
pub struct Doll {
    name: &'static str,
    item: &'static str,
    sprite: &'static str,
    held: &'static str,
    slots: usize,
    look: model::Look,
}

impl PartialEq for Doll {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

/// Each doll's slots: one row more than the original's (Russia and Moscow, with their own, aside).
const DOLL_SLOTS: usize = INVENTORY + 9;
const EXTRA_SLOTS: usize = 27;

const fn base(name: &'static str, item: &'static str, sprite: &'static str, held: &'static str) -> Doll {
    Doll { name, item, sprite, held, slots: DOLL_SLOTS, look: model::Look::Base }
}

static DOLLS: [Doll; 17] = [
    base(goals::SHANGHAI, "alice_dolls:shanghai_doll", "alice_dolls:entity/shanghai", "minecraft:golden_sword"),
    base(goals::HORAI, "alice_dolls:horai_doll", "alice_dolls:entity/horai", "minecraft:golden_pickaxe"),
    base(goals::DIAMOND_HORAI, "alice_dolls:diamond_horai_doll", "alice_dolls:entity/horai", "minecraft:diamond_pickaxe"),
    base(goals::NETHERITE_HORAI, "alice_dolls:netherite_horai_doll", "alice_dolls:entity/horai", "minecraft:netherite_pickaxe"),
    base(goals::OOEDO, "alice_dolls:ooedo_doll", "alice_dolls:entity/ooedo", "minecraft:tnt"),
    base(goals::GERMANY, "alice_dolls:germany_doll", "alice_dolls:entity/germany", "minecraft:golden_axe"),
    base(goals::LONDON, "alice_dolls:london_doll", "alice_dolls:entity/london", "minecraft:ender_eye"),
    // Russia carries four rows.
    Doll { slots: 36, ..base(goals::RUSSIA, "alice_dolls:russia_doll", "alice_dolls:entity/russia", "minecraft:chest") },
    // Moscow: Russia set in diamonds, with two rows more, stacks of 128, and a doll to look after.
    Doll { slots: 54, ..base(goals::MOSCOW, "alice_dolls:moscow_doll", "alice_dolls:entity/russia", "minecraft:ender_chest") },
    base(goals::GRAND_GUIGNOL, "alice_dolls:grand_guignol_doll", "alice_dolls:entity/grandguignol", ""),
    // The extras carry two rows. Their items were thKaguya's: Lunatic has Aya's fan, Yuuka's
    // parasol and a gohei (its Miko Stick); Marisa's Mini-Hakkero is this mod's own.
    Doll {
        name: goals::MARISA,
        item: "alice_dolls:marisa_doll",
        sprite: "alice_dolls:entity/marisa",
        held: "thkaguyamod:mini_hakkero",
        slots: EXTRA_SLOTS,
        look: model::Look::Boxes(&model::MARISA),
    },
    Doll {
        name: goals::REIMU,
        item: "alice_dolls:reimu_doll",
        sprite: "alice_dolls:entity/reimu",
        held: "lunatic:miko_stick",
        slots: EXTRA_SLOTS,
        look: model::Look::Boxes(&model::REIMU),
    },
    Doll {
        name: goals::AYA,
        item: "alice_dolls:aya_doll",
        sprite: "alice_dolls:entity/aya",
        held: "lunatic:hauchiwa_fan",
        slots: EXTRA_SLOTS,
        look: model::Look::Boxes(&model::AYA),
    },
    Doll {
        name: goals::WAKASAGI,
        item: "alice_dolls:wakasagi_doll",
        sprite: "alice_dolls:entity/wakasagi",
        held: "minecraft:fishing_rod",
        slots: EXTRA_SLOTS,
        look: model::Look::Boxes(&model::WAKASAGI),
    },
    Doll {
        name: goals::GOLIATH,
        item: "alice_dolls:goliath_doll",
        sprite: "alice_dolls:entity/goliath",
        held: "minecraft:diamond_sword",
        // The first four are what it wears (`goals::wear_armour`), the rest a row and a half.
        slots: EXTRA_SLOTS,
        look: model::Look::Boxes(&model::GOLIATH),
    },
    // Goliath with a netherite sword: two harder, quicker, and 5% more health.
    Doll {
        name: goals::NETHERITE_GOLIATH,
        item: "alice_dolls:netherite_goliath_doll",
        sprite: "alice_dolls:entity/goliath",
        held: "minecraft:netherite_sword",
        slots: EXTRA_SLOTS,
        look: model::Look::Boxes(&model::GOLIATH),
    },
    Doll {
        name: goals::YUUKA,
        item: "alice_dolls:yuuka_doll",
        sprite: "alice_dolls:entity/yuuka",
        held: "lunatic:yuuka_parasol",
        slots: EXTRA_SLOTS,
        look: model::Look::Boxes(&model::YUUKA),
    },
];

fn doll_named(name: &str) -> Option<&'static Doll> {
    DOLLS.iter().find(|d| d.name == name)
}

fn doll_for_item(item: &str) -> Option<&'static Doll> {
    DOLLS.iter().find(|d| d.item == item)
}

/// Which doll the mob `id` is, by its name.
fn doll_of(host: &dyn Host, id: u32) -> Option<&'static Doll> {
    let name = host.entity(id).and_then(|e| host.entity_name(e))?;
    doll_named(&name)
}

/// Half as much again as a player's 20: `goals.rs` keeps every doll at this; Goliath has 66 and the
/// Netherite Goliath 69.3.
const HEALTH: f32 = 30.0;

fn health_of(doll: &Doll) -> f32 {
    match doll.name {
        goals::GOLIATH => 66.0,
        goals::NETHERITE_GOLIATH => 69.3,
        _ => HEALTH,
    }
}
/// The original's starting state bits, 528: chat level 2.
const DEFAULT_CHAT_LEVEL: u8 = 2;

pub fn hooks() -> Box<dyn Hooks> {
    Box::new(Mod)
}

struct Mod;

/// What is saved with a doll beyond the game's own data.
#[derive(Clone, Debug, PartialEq)]
struct DollData {
    /// The player's UUID.
    owner: u128,
    mode: u8,
    /// Work AI on or off.
    enabled: bool,
    chat_level: u8,
    /// Its inventory, by slot.
    items: Vec<Option<ItemStack>>,
    /// A Moscow doll's doll: its UUID, 0 for none.
    bound: u128,
    /// A Diamond or Netherite Horai keeps the stone and earth it digs, rather than voiding it.
    keep_rock: bool,
    /// Its baubles: the 4x4 grid, `goals::BAUBLE_SLOTS` of them.
    baubles: Vec<Option<ItemStack>>,
    /// The anchor it keeps to on patrol: the nearest within 30 blocks when it was placed or set to
    /// patrol.
    anchor: Option<[i32; 3]>,
    /// Experience points it has picked up, for its owner to take with a glass bottle.
    xp: u32,
    /// Favour points, from food it ate and Point items it was given: its level comes from them.
    favour: u32,
    /// The level it was at when it took its present form: a Horai or Goliath grows into its next
    /// form three levels on.
    form_level: u32,
}

impl Default for DollData {
    fn default() -> Self {
        Self {
            owner: 0,
            mode: goals::STANDBY,
            enabled: true,
            chat_level: DEFAULT_CHAT_LEVEL,
            items: vec![None; INVENTORY],
            bound: 0,
            keep_rock: false,
            baubles: vec![None; goals::BAUBLE_SLOTS],
            xp: 0,
            favour: 0,
            form_level: 0,
            anchor: None,
        }
    }
}

fn encode(data: &DollData) -> Result<Compound, AttachmentError> {
    let mut tag: Compound = [
        ("owner_high".into(), Tag::Long((data.owner >> 64) as i64)),
        ("owner_low".into(), Tag::Long(data.owner as u64 as i64)),
        ("mode".into(), Tag::Byte(data.mode as i8)),
        ("enabled".into(), Tag::Byte(i8::from(data.enabled))),
        ("chat_level".into(), Tag::Byte(data.chat_level as i8)),
        ("Items".into(), goals::encode_items(&split_big(&data.items))),
        ("bound_high".into(), Tag::Long((data.bound >> 64) as i64)),
        ("bound_low".into(), Tag::Long(data.bound as u64 as i64)),
        ("keep_rock".into(), Tag::Byte(i8::from(data.keep_rock))),
        ("Baubles".into(), goals::encode_items(&data.baubles)),
    ]
    .into();
    if let Some(a) = data.anchor {
        tag.insert("anchor".into(), Tag::IntArray(a.to_vec()));
    }
    if data.xp > 0 {
        tag.insert("xp".into(), Tag::Int(data.xp.min(i32::MAX as u32) as i32));
    }
    if data.favour > 0 {
        tag.insert("favour".into(), Tag::Int(data.favour.min(i32::MAX as u32) as i32));
    }
    if data.form_level > 0 {
        tag.insert("form_level".into(), Tag::Int(data.form_level as i32));
    }
    Ok(tag)
}

/// A grid saved by the 4x4 version, into the 3x3: the first nine kept, the rest into empty inventory
/// slots (and lost only when there are none).
fn fit_baubles(data: &mut DollData) {
    let worn: Vec<ItemStack> = data.baubles.drain(..).flatten().collect();
    data.baubles = vec![None; goals::BAUBLE_SLOTS];
    for (i, b) in worn.into_iter().enumerate() {
        if i < goals::BAUBLE_SLOTS {
            data.baubles[i] = Some(b);
        } else if let Some(slot) = data.items.iter_mut().find(|s| s.is_none()) {
            *slot = Some(b);
        }
    }
}

fn decode(tag: &Compound) -> Result<DollData, AttachmentError> {
    let long = |key: &str| match tag.get(&key.into()) {
        Some(Tag::Long(n)) => Ok(*n as u64),
        _ => Err(AttachmentError::Decode),
    };
    let byte = |key: &str| match tag.get(&key.into()) {
        Some(Tag::Byte(n)) => Some(*n as u8),
        _ => None,
    };
    // A doll saved by the first version has only `standby`, and otherwise followed.
    let old_mode = if byte("standby") == Some(1) { goals::STANDBY } else { goals::FOLLOW };
    let mut data = DollData {
        owner: (u128::from(long("owner_high")?) << 64) | u128::from(long("owner_low")?),
        mode: byte("mode").unwrap_or(old_mode),
        enabled: byte("enabled").is_none_or(|n| n != 0),
        chat_level: byte("chat_level").unwrap_or(DEFAULT_CHAT_LEVEL),
        // Every slot a doll can have (a slot past them was lost before), and big stacks put back.
        items: merge_big(goals::decode_items(tag.get(&"Items".into()), 256)),
        bound: match (long("bound_high"), long("bound_low")) {
            (Ok(h), Ok(l)) => (u128::from(h) << 64) | u128::from(l),
            _ => 0,
        },
        keep_rock: byte("keep_rock").is_some_and(|n| n != 0),
        baubles: goals::decode_items(tag.get(&"Baubles".into()), 16),
        anchor: match tag.get(&"anchor".into()) {
            Some(Tag::IntArray(v)) if v.len() == 3 => Some([v[0], v[1], v[2]]),
            _ => None,
        },
        xp: match tag.get(&"xp".into()) {
            Some(Tag::Int(n)) => (*n).max(0) as u32,
            _ => 0,
        },
        favour: match tag.get(&"favour".into()) {
            Some(Tag::Int(n)) => (*n).max(0) as u32,
            _ => 0,
        },
        form_level: match tag.get(&"form_level".into()) {
            Some(Tag::Int(n)) => (*n).max(0) as u32,
            _ => 0,
        },
    };
    fit_baubles(&mut data);
    Ok(data)
}

/// How far round a doll it draws experience orbs in.
const XP_REACH: f64 = 4.0;

/// Takes in the experience orbs round doll `id`: whether it took any.
fn absorb_xp(host: &mut dyn Host, id: u32, data: &mut DollData) -> bool {
    let Some(entity) = host.entity(id) else { return false };
    let (Some(at), Some(level)) = (host.entity_position(entity), host.entity_level(entity)) else { return false };
    let dimension = level.dimension().to_owned();
    let min = [at[0] - XP_REACH, at[1] - 2.0, at[2] - XP_REACH];
    let max = [at[0] + XP_REACH, at[1] + 3.0, at[2] + XP_REACH];
    let mut took = false;
    for orb in host.experience_orbs(&dimension, min, max) {
        let points = orb.value.saturating_mul(orb.count.max(1));
        // Gone at once, or, if this game won't remove it, aged out on its next tick.
        if host.remove_entity(&dimension, orb.id) || host.set_experience_orb(&dimension, orb.id, None, None, Some(6000)) {
            data.xp = data.xp.saturating_add(points);
            took = true;
        }
    }
    if took {
        host.level_play_sound(level, "minecraft:entity.experience_orb.pickup", at, 6, 0.3, 1.4);
    }
    took
}

/// `toggleMode`.
fn is_goliath(name: &str) -> bool {
    name == goals::GOLIATH || name == goals::NETHERITE_GOLIATH
}

/// The next mode for `doll`: a Goliath is too big for its owner's head, and skips Rideon.
fn next_mode_of(name: &str, mode: u8) -> u8 {
    let next = next_mode(mode);
    if next == goals::RIDE && is_goliath(name) { goals::STANDBY } else { next }
}

fn next_mode(mode: u8) -> u8 {
    match mode {
        goals::STANDBY => goals::PATROL,
        goals::PATROL => goals::FOLLOW,
        goals::FOLLOW => goals::RIDE,
        _ => goals::STANDBY,
    }
}

fn mode_name(mode: u8) -> &'static str {
    match mode {
        goals::PATROL => "Patrol mode",
        goals::FOLLOW => "Follow mode",
        goals::RIDE => "Rideon mode",
        goals::SECRET => "Secret mode: off to find diamonds and ancient debris",
        goals::SECRET_ALL => "Secret mode, every ore: iron, coal, gold, lapis, redstone and the rest",
        _ => "Standby mode",
    }
}

/// `chatMessage`'s `lastMsg`: a doll does not repeat itself.
static LAST_SAID: LazyLock<Mutex<HashMap<u32, String>>> = LazyLock::new(Default::default);

/// `chatMessage`: tells the owner, if the doll's chat level allows and it is news.
fn say(host: &mut dyn Host, id: u32, data: &DollData, level: u8, text: &str) {
    if data.chat_level < level || goals::wears(id, "alice_dolls:mute_bauble") {
        return;
    }
    let Ok(mut last) = LAST_SAID.lock() else { return };
    if last.get(&id).is_some_and(|said| said.eq_ignore_ascii_case(text)) {
        return;
    }
    last.insert(id, text.to_owned());
    drop(last);
    let name = doll_of(host, id).map_or("Doll", |d| d.name);
    host.tell(data.owner, Text::literal(format!("{name} : {text}")));
}

fn click(host: &mut dyn Host, id: u32) {
    if let Some(at) = host.entity(id).and_then(|e| host.entity_position(e)) {
        host.play_local_sound("minecraft:ui.button.click", at, 0.3, 1.0, 0);
    }
}

fn tell_state(id: u32, data: &DollData) {
    goals::set_state(id, data.mode, data.enabled, data.owner);
}

static DOLL: AttachmentType<DollData> = AttachmentType {
    namespace: "alice_dolls",
    name: "doll",
    default: DollData::default,
    codec: Some(AttachmentCodec { encode, decode }),
    copy_on_death: false,
    sync: false,
};

/// What is remembered of each doll seen since the game started: a dying or exploding doll's item
/// and inventory go to its owner, and by then its attachment may be gone.
#[derive(Clone)]
struct Known {
    doll: &'static Doll,
    owner: u128,
    items: Vec<Option<ItemStack>>,
    baubles: Vec<Option<ItemStack>>,
    favour: u32,
    form_level: u32,
}

static KNOWN: LazyLock<Mutex<HashMap<u32, Known>>> = LazyLock::new(Default::default);

fn remember(id: u32, doll: &'static Doll, data: &DollData) {
    if let Ok(mut known) = KNOWN.lock() {
        known.insert(
            id,
            Known {
                doll,
                owner: data.owner,
                items: data.items.clone(),
                baubles: data.baubles.clone(),
                favour: data.favour,
                form_level: data.form_level,
            },
        );
    }
}

fn known(id: u32) -> Option<Known> {
    KNOWN.lock().ok()?.get(&id).cloned()
}

fn forget(id: u32) -> Option<Known> {
    KNOWN.lock().ok()?.remove(&id)
}

/// One right-click reaches the game more than once: each hand asks, and a held button asks again
/// every 4 ticks. A player's click counts only after a pause of `CLICK_GAP` since the last time they
/// asked, so holding the button is one click however long it is held.
static LAST_CLICK: LazyLock<Mutex<HashMap<u128, Instant>>> = LazyLock::new(Default::default);
const CLICK_GAP: Duration = Duration::from_millis(350);

fn fresh_click(player: u128) -> bool {
    let Ok(mut clicks) = LAST_CLICK.lock() else { return true };
    let now = Instant::now();
    let fresh = clicks.get(&player).is_none_or(|at| now.duration_since(*at) >= CLICK_GAP);
    clicks.insert(player, now);
    fresh
}

/// Hands `stacks` to `owner`: what did not fit, or everything when the owner is not here.
fn give_to_owner(host: &mut dyn Host, owner: u128, stacks: Vec<ItemStack>) -> Vec<ItemStack> {
    if owner == 0 {
        return stacks;
    }
    stacks.into_iter().filter_map(|stack| host.give_stack(owner, stack)).collect()
}

/// A gone doll's item, for its owner: it carries the doll's inventory as a shulker box does.
fn belongings(k: &Known) -> Vec<ItemStack> {
    let mut item = doll_item(k.doll, &k.items);
    // Its baubles ride in the item's container from slot `BAUBLES_IN_ITEM`, past every inventory.
    let worn: Vec<(u8, ItemStack)> =
        k.baubles.iter().enumerate().filter_map(|(i, s)| Some(((BAUBLES_IN_ITEM + i) as u8, s.clone()?))).collect();
    if !worn.is_empty() {
        let data = item.extra.get_or_insert_with(|| Box::new(StackData::default()));
        data.container.extend(worn);
    }
    if k.favour > 0 {
        let data = item.extra.get_or_insert_with(|| Box::new(StackData::default()));
        data.container.push((FAVOUR_IN_ITEM, favour_marker(k.favour, k.form_level)));
    }
    vec![item]
}

/// Where a doll item keeps its doll's favour, in its container: a named marker past its baubles.
const FAVOUR_IN_ITEM: u8 = 250;

fn favour_marker(favour: u32, form_level: u32) -> ItemStack {
    let mut s = ItemStack::new("minecraft:paper", 1);
    s.extra = Some(Box::new(StackData {
        custom_name: Some(Tag::String(format!("favour:{favour}:{form_level}").as_str().into())),
        ..StackData::default()
    }));
    s
}

/// The favour points and form level a doll item carries.
fn favour_in(stack: &ItemStack) -> (u32, u32) {
    for (i, s) in stack.container() {
        if *i != FAVOUR_IN_ITEM {
            continue;
        }
        if let Some(Tag::String(name)) = s.extra.as_deref().and_then(|d| d.custom_name.as_ref()) {
            let name = name.to_string_lossy();
            let mut parts = name.strip_prefix("favour:").unwrap_or("").split(':');
            let favour = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
            let form = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
            return (favour, form);
        }
    }
    (0, 0)
}

// ---- Favour ----

/// Each doll's favour as its name tag shows it: level, points into it, points it takes. Written by
/// the server's tick and read by the client's (single player shares the process).
static FAVOUR_SHOWN: LazyLock<Mutex<HashMap<u32, (u32, u32, u32)>>> = LazyLock::new(Default::default);

/// Its name with its favour level and a bar of how far it is into the next: "Shanghai Doll  Lv 2
/// ■■■■□□□□□□ 41%".
fn favour_tag(name: &str, level: u32, into: u32, need: u32) -> String {
    let fraction = if need == 0 { 0.0 } else { f64::from(into) / f64::from(need) };
    let filled = ((fraction * 10.0).floor() as usize).min(10);
    let bar: String = "■".repeat(filled) + &"□".repeat(10 - filled);
    format!("{name}  Lv {level} {bar} {}%", (fraction * 100.0).floor() as u32)
}

/// Points to reach level 1; each level after takes three quarters more than the one before.
const FAVOUR_FIRST: f64 = 100.0;
const FAVOUR_GROWTH: f64 = 1.75;
/// What one Point item is worth.
const POINT_FAVOUR: u32 = 5;

fn cooked(item: &str) -> Option<&'static str> {
    let short = item.strip_prefix("minecraft:")?;
    Some(match short {
        "beef" => "minecraft:cooked_beef",
        "porkchop" => "minecraft:cooked_porkchop",
        "chicken" => "minecraft:cooked_chicken",
        "mutton" => "minecraft:cooked_mutton",
        "rabbit" => "minecraft:cooked_rabbit",
        "cod" => "minecraft:cooked_cod",
        "salmon" => "minecraft:cooked_salmon",
        "potato" => "minecraft:baked_potato",
        "kelp" => "minecraft:dried_kelp",
        "chorus_fruit" => "minecraft:popped_chorus_fruit",
        "iron_ore" | "deepslate_iron_ore" | "raw_iron" => "minecraft:iron_ingot",
        "gold_ore" | "deepslate_gold_ore" | "nether_gold_ore" | "raw_gold" => "minecraft:gold_ingot",
        "copper_ore" | "deepslate_copper_ore" | "raw_copper" => "minecraft:copper_ingot",
        "coal_ore" | "deepslate_coal_ore" => "minecraft:coal",
        "diamond_ore" | "deepslate_diamond_ore" => "minecraft:diamond",
        "emerald_ore" | "deepslate_emerald_ore" => "minecraft:emerald",
        "lapis_ore" | "deepslate_lapis_ore" => "minecraft:lapis_lazuli",
        "redstone_ore" | "deepslate_redstone_ore" => "minecraft:redstone",
        "nether_quartz_ore" => "minecraft:quartz",
        "ancient_debris" => "minecraft:netherite_scrap",
        "sand" | "red_sand" => "minecraft:glass",
        "cobblestone" => "minecraft:stone",
        "stone" => "minecraft:smooth_stone",
        "cobbled_deepslate" => "minecraft:deepslate",
        "stone_bricks" => "minecraft:cracked_stone_bricks",
        "deepslate_bricks" => "minecraft:cracked_deepslate_bricks",
        "deepslate_tiles" => "minecraft:cracked_deepslate_tiles",
        "nether_bricks" => "minecraft:cracked_nether_bricks",
        "polished_blackstone_bricks" => "minecraft:cracked_polished_blackstone_bricks",
        "sandstone" => "minecraft:smooth_sandstone",
        "red_sandstone" => "minecraft:smooth_red_sandstone",
        "quartz_block" => "minecraft:smooth_quartz",
        "basalt" => "minecraft:smooth_basalt",
        "clay_ball" => "minecraft:brick",
        "clay" => "minecraft:terracotta",
        "netherrack" => "minecraft:nether_brick",
        "cactus" => "minecraft:green_dye",
        "sea_pickle" => "minecraft:lime_dye",
        "wet_sponge" => "minecraft:sponge",
        "chorus_flower" => "minecraft:popped_chorus_fruit",
        "resin_clump" => "minecraft:resin_brick",
        s if s.ends_with("_log") || s.ends_with("_wood") || s.ends_with("_stem") || s.ends_with("_hyphae") => {
            "minecraft:charcoal"
        }
        s if s.ends_with("_terracotta") && !s.ends_with("glazed_terracotta") && s != "terracotta" => {
            return glazed(s);
        }
        _ => return None,
    })
}

/// A coloured terracotta's glazed kind.
fn glazed(short: &str) -> Option<&'static str> {
    const COLOURS: [&str; 16] = [
        "white", "orange", "magenta", "light_blue", "yellow", "lime", "pink", "gray", "light_gray", "cyan", "purple",
        "blue", "brown", "green", "red", "black",
    ];
    const GLAZED: [&str; 16] = [
        "minecraft:white_glazed_terracotta", "minecraft:orange_glazed_terracotta", "minecraft:magenta_glazed_terracotta",
        "minecraft:light_blue_glazed_terracotta", "minecraft:yellow_glazed_terracotta", "minecraft:lime_glazed_terracotta",
        "minecraft:pink_glazed_terracotta", "minecraft:gray_glazed_terracotta", "minecraft:light_gray_glazed_terracotta",
        "minecraft:cyan_glazed_terracotta", "minecraft:purple_glazed_terracotta", "minecraft:blue_glazed_terracotta",
        "minecraft:brown_glazed_terracotta", "minecraft:green_glazed_terracotta", "minecraft:red_glazed_terracotta",
        "minecraft:black_glazed_terracotta",
    ];
    let colour = short.strip_suffix("_terracotta")?;
    COLOURS.iter().position(|c| *c == colour).map(|i| GLAZED[i])
}

/// Within this of the camera, a doll riding on a head is not drawn (first person); in third person
/// the camera is further off and it is.
const RIDE_HIDE_RANGE: f64 = 1.5;

const COOK_TICKS: u32 = 200;
static COOK_TICKS_LEFT: LazyLock<Mutex<HashMap<u32, u32>>> = LazyLock::new(Default::default);

fn cook(host: &mut dyn Host, id: u32) {
    let due = COOK_TICKS_LEFT.lock().is_ok_and(|mut t| {
        let n = t.entry(id).or_default();
        *n = (*n + 1) % COOK_TICKS;
        *n == 0
    });
    if !due || goals::chest_open() == Some(id) {
        return;
    }
    let holder = Holder::Entity(id);
    let mut data = DOLL.get(host, holder).unwrap_or_default();
    let mut items = goals::inventory(id).unwrap_or_else(|| data.items.clone());
    let Some((slot, result)) = items
        .iter()
        .enumerate()
        .find_map(|(i, s)| s.as_ref().filter(|s| s.count > 0 && s.extra.is_none()).and_then(|s| Some((i, cooked(&s.item)?))))
    else {
        return;
    };
    let out = ItemStack::new(result, 1);
    let max = host.max_stack_size(result).unwrap_or(64);
    // Room for what it makes, counting the slot the last of the raw item leaves empty.
    let last = items[slot].as_ref().is_some_and(|s| s.count == 1);
    if room_for(&items, &out, max) == 0 && !last {
        return;
    }
    if let Some(s) = items[slot].as_mut() {
        s.count -= 1;
        if s.count == 0 {
            items[slot] = None;
        }
    }
    store_some(&mut items, &out, max);
    data.items = items;
    goals::put_inventory(id, data.items.clone());
    if let Some(doll) = doll_of(host, id) {
        remember(id, doll, &data);
    }
    if let Some(at) = host.entity(id).and_then(|e| host.entity_position(e)) {
        host.play_local_sound("minecraft:block.furnace.fire_crackle", at, 0.5, 1.0, 0);
    }
    let _ = DOLL.set(host, holder, data);
}

/// The head a mob of `kind` leaves when a doll kills it: only the mobs that have one in the game.
fn mob_head(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "minecraft:zombie" => "minecraft:zombie_head",
        "minecraft:skeleton" => "minecraft:skeleton_skull",
        "minecraft:creeper" => "minecraft:creeper_head",
        "minecraft:wither_skeleton" => "minecraft:wither_skeleton_skull",
        "minecraft:piglin" => "minecraft:piglin_head",
        "minecraft:ender_dragon" => "minecraft:dragon_head",
        _ => return None,
    })
}

/// The favour each Touhou point or power item is worth to a doll.
fn point_favour(item: &str) -> Option<u32> {
    Some(match item.strip_prefix("thkaguyamod:")? {
        "point_item" => POINT_FAVOUR,
        "small_power_item" => 3,
        "star_item" | "faith_item" | "cherry_item" => 8,
        "ufo_red" | "ufo_blue" | "ufo_green" => 10,
        "big_power_item" => 15,
        "full_power_item" => 40,
        _ => return None,
    })
}
/// How many levels a Horai or Goliath takes to grow into its next form.
const LEVELS_PER_FORM: u32 = 3;

/// The points the step from `level` to `level + 1` takes.
fn favour_step(level: u32) -> u32 {
    (FAVOUR_FIRST * FAVOUR_GROWTH.powi(level.min(60) as i32)).round().min(u32::MAX as f64) as u32
}

/// The level `favour` points make, the points into it, and the points the next one takes.
fn favour_level(favour: u32) -> (u32, u32, u32) {
    let (mut level, mut left) = (0, favour);
    loop {
        let step = favour_step(level);
        if left < step {
            return (level, left, step);
        }
        left -= step;
        level += 1;
    }
}

/// The form a doll grows into, if it has one.
fn next_form(name: &str) -> Option<&'static str> {
    match name {
        goals::HORAI => Some(goals::DIAMOND_HORAI),
        goals::DIAMOND_HORAI => Some(goals::NETHERITE_HORAI),
        goals::SHANGHAI => Some(goals::GOLIATH),
        goals::GOLIATH => Some(goals::NETHERITE_GOLIATH),
        goals::RUSSIA => Some(goals::MOSCOW),
        _ => None,
    }
}

/// `doll`'s item with `items` inside it, slot by slot (`minecraft:container`, a shulker box's).
fn doll_item(doll: &Doll, items: &[Option<ItemStack>]) -> ItemStack {
    let mut stack = ItemStack::new(doll.item, 1);
    let container: Vec<(u8, ItemStack)> = split_big(items)
        .iter()
        .enumerate()
        .filter_map(|(i, s)| Some((u8::try_from(i).ok()?, s.clone()?)))
        .collect();
    if !container.is_empty() {
        stack.extra = Some(Box::new(StackData { container, ..StackData::default() }));
    }
    stack
}

/// What a doll item carries, as `slots` slots: a stack in a slot past them takes the first empty one,
/// and is lost only when none is.
/// Where a doll item keeps its doll's baubles, in its container.
const BAUBLES_IN_ITEM: usize = 200;

/// The baubles a doll item carries.
fn baubles_in(stack: &ItemStack) -> Vec<Option<ItemStack>> {
    let mut b = vec![None; goals::BAUBLE_SLOTS];
    for (i, s) in stack.container() {
        if let Some(slot) = usize::from(*i).checked_sub(BAUBLES_IN_ITEM).and_then(|j| b.get_mut(j)) {
            *slot = Some(s.clone());
        }
    }
    b
}

fn items_in(stack: &ItemStack, slots: usize) -> Vec<Option<ItemStack>> {
    let mut raw = vec![None; BAUBLES_IN_ITEM];
    for (i, s) in stack.container() {
        if usize::from(*i) < BAUBLES_IN_ITEM {
            raw[usize::from(*i)] = Some(s.clone());
        }
    }
    let mut items = merge_big(raw);
    let over: Vec<ItemStack> =
        if items.len() > slots { items.split_off(slots).into_iter().flatten().collect() } else { Vec::new() };
    items.resize(slots, None);
    for s in over {
        if let Some(slot) = items.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(s);
        }
    }
    items
}

/// Item NBT keeps a count only up to 99: a bigger stack (a Moscow doll's) is saved as 99 in its slot
/// and the rest in 99s from `EXTRA_SLOT` on, past every doll's slots, and put together when read.
const EXTRA_SLOT: usize = 64;

fn split_big(items: &[Option<ItemStack>]) -> Vec<Option<ItemStack>> {
    let mut out: Vec<Option<ItemStack>> = items
        .iter()
        .map(|s| {
            s.clone().map(|mut s| {
                s.count = s.count.min(99);
                s
            })
        })
        .collect();
    let mut extra = Vec::new();
    for s in items.iter().flatten() {
        let mut left = s.count.saturating_sub(99);
        while left > 0 {
            let mut e = s.clone();
            e.count = left.min(99);
            left -= e.count;
            extra.push(Some(e));
        }
    }
    if !extra.is_empty() {
        out.resize(EXTRA_SLOT.max(out.len()), None);
        out.extend(extra);
    }
    out.truncate(256);
    out
}

fn merge_big(mut items: Vec<Option<ItemStack>>) -> Vec<Option<ItemStack>> {
    let extra: Vec<ItemStack> =
        if items.len() > EXTRA_SLOT { items.split_off(EXTRA_SLOT).into_iter().flatten().collect() } else { Vec::new() };
    for e in extra {
        if let Some(s) = items.iter_mut().flatten().find(|s| same_kind(s, &e)) {
            s.count += e.count;
        } else if let Some(slot) = items.iter_mut().find(|s| s.is_none()) {
            *slot = Some(e);
        }
    }
    while items.len() > INVENTORY && items.last().is_some_and(Option::is_none) {
        items.pop();
    }
    items
}

/// Takes the one doll item `stack` the player placed from their hand: a plain one as any item, one
/// carrying an inventory from the very slot, since it stacks with nothing else. Whether it was taken.
///
/// Takes one of the doll item the player is placing out of their hand. While they act, the game
/// holds their state swapped in, so the hub's copy of their inventory is stale (a swap against it
/// always failed, and dolls holding items wouldn't place): the hand is read and set on the player
/// themself, then checked. The stack taken, one of it, is returned.
fn take_placed(host: &mut dyn Host, player: &NativeServerPlayer, stack: &ItemStack) -> Option<ItemStack> {
    if stack.extra.is_none() {
        return (host.take_items(&stack.item, 1) == 1).then(|| stack.clone());
    }
    // The player's own slots, as the game holds them while they act (`take_items` takes only plain
    // items, and the equipment calls read a stale copy): one off the slot holding this very stack.
    // A count of 0 is emptied at the slot's next tick.
    for slot in (0..36).chain([-106]) {
        let Some(s) = host.carried_stack(slot) else { continue };
        if s.item == stack.item && s.extra == stack.extra && s.count > 0 {
            s.count -= 1;
            let mut one = stack.clone();
            one.count = 1;
            return Some(one);
        }
    }
    debug(host, &format!("  {}: not in any carried slot", stack.item));
    let living = u32::try_from(player.entity_id).ok().and_then(|id| host.entity(id)).and_then(|e| host.living(e));
    for slot in [EquipmentSlot::MainHand, EquipmentSlot::OffHand] {
        let Some(actual) = host.equipment(slot).filter(|s| s.item == stack.item) else { continue };
        let mut rest = actual.clone();
        rest.count -= 1;
        let rest = (rest.count > 0).then_some(rest);
        let Some(living) = living else { break };
        if living.set_equipment(host, slot, rest.clone()) && host.equipment(slot) == rest {
            let mut one = actual;
            one.count = 1;
            return Some(one);
        }
        debug(host, &format!("  {}: setting the {slot:?} slot did not take", stack.item));
    }
    // Last: one of that doll item from wherever it is, with the contents the click carried.
    debug(host, &format!("  {}: taking it by its id instead", stack.item));
    (host.take_items(&stack.item, 1) == 1).then(|| {
        let mut one = stack.clone();
        one.count = 1;
        one
    })
}

/// `minecraft:iron_ingot` as "Iron Ingot", for a doll item's tooltip.
fn item_label(item: &str) -> String {
    let path = item.rsplit(':').next().unwrap_or(item);
    path.split('_')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            c.next().map_or_else(String::new, |f| f.to_uppercase().chain(c).collect())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// How many of a doll item's stacks its tooltip lists, as a shulker box's lists five.
const TOOLTIP_STACKS: usize = 5;

/// A line in `alice_dolls-debug.log` in the game folder: what happened when a doll item was used,
/// to find out why some will not place.
fn debug(host: &(impl Host + ?Sized), line: &str) {
    use std::io::Write;
    let path = host.instance_dir().join("alice_dolls-debug.log");
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let _ = writeln!(file, "{secs} {line}");
    }
}

/// Mobs already written to the debug log, so each shows once a session.
static RENDER_SEEN: LazyLock<Mutex<std::collections::HashSet<u32>>> = LazyLock::new(Default::default);
static TICK_SEEN: LazyLock<Mutex<std::collections::HashSet<u32>>> = LazyLock::new(Default::default);

fn first_time(seen: &Mutex<std::collections::HashSet<u32>>, id: u32) -> bool {
    seen.lock().is_ok_and(|mut seen| seen.insert(id))
}

/// The data a doll is summoned with.
fn doll_nbt(doll: &Doll) -> Compound {
    let max_health: Compound = [
        ("id".into(), Tag::String("minecraft:max_health".into())),
        ("base".into(), Tag::Double(f64::from(health_of(doll)).to_bits())),
    ]
    .into();
    [
        ("CustomName".into(), Tag::String(doll.name.into())),
        ("Silent".into(), Tag::Byte(1)),
        ("PersistenceRequired".into(), Tag::Byte(1)),
        // A doll lays no eggs: about three years of play until the first.
        ("EggLayTime".into(), Tag::Int(i32::MAX)),
        ("Health".into(), Tag::Float(health_of(doll).to_bits())),
        ("attributes".into(), Tag::List(vec![Tag::Compound(max_health)])),
    ]
    .into()
}

// ---- Inventories ----

/// The doll inventory each player has open: the doll, and the menu's window id.
static OPEN: LazyLock<Mutex<HashMap<u128, (u32, i32)>>> = LazyLock::new(Default::default);

fn same_kind(a: &ItemStack, b: &ItemStack) -> bool {
    // Empty extra data is none: a stack that has been through a menu may carry it.
    fn data(s: &ItemStack) -> Option<&StackData> {
        s.extra.as_deref().filter(|d| **d != StackData::default())
    }
    a.item == b.item && a.damage == b.damage && data(a) == data(b)
}

/// How many of `stack` would fit into `items`.
fn room_for(items: &[Option<ItemStack>], stack: &ItemStack, max: u32) -> u32 {
    items
        .iter()
        .map(|slot| match slot {
            None => max,
            Some(s) if same_kind(s, stack) => max.saturating_sub(s.count),
            Some(_) => 0,
        })
        .sum()
}

/// Puts as much of `stack` into `items` as fits: how many are left over.
fn store_some(items: &mut [Option<ItemStack>], stack: &ItemStack, max: u32) -> u32 {
    let mut left = stack.count;
    for slot in items.iter_mut().filter(|s| s.is_some()) {
        if let Some(s) = slot
            && same_kind(s, stack)
        {
            let add = left.min(max.saturating_sub(s.count));
            s.count += add;
            left -= add;
        }
    }
    for slot in items.iter_mut() {
        if left == 0 {
            break;
        }
        if slot.is_none() {
            let mut s = stack.clone();
            s.count = left.min(max);
            left -= s.count;
            *slot = Some(s);
        }
    }
    left
}

/// The Goliath (either kind) the player is looking at within five blocks: the body it draws, a
/// block across and 2.75 tall, not the chicken's box it stands on.
fn goliath_in_sight(host: &mut dyn Host, player: &NativeServerPlayer) -> Option<u32> {
    let (yaw, pitch) = (f64::from(player.yaw).to_radians(), f64::from(player.pitch).to_radians());
    let dir = [-yaw.sin() * pitch.cos(), -pitch.sin(), yaw.cos() * pitch.cos()];
    let eye = [player.position[0], player.position[1] + f64::from(player.eye_height), player.position[2]];
    let mut best: Option<(f64, u32)> = None;
    for (id, kind, at) in host.level_entities(&player.dimension) {
        if kind != KIND || !doll_of(host, id).is_some_and(|d| d.name == goals::GOLIATH || d.name == goals::NETHERITE_GOLIATH) {
            continue;
        }
        let lo = [at[0] - 0.6, at[1], at[2] - 0.6];
        let hi = [at[0] + 0.6, at[1] + 2.75, at[2] + 0.6];
        let (mut near, mut far) = (0.0f64, 5.0f64);
        let mut hit = true;
        for k in 0..3 {
            if dir[k].abs() < 1e-9 {
                if eye[k] < lo[k] || eye[k] > hi[k] {
                    hit = false;
                }
            } else {
                let (t0, t1) = ((lo[k] - eye[k]) / dir[k], (hi[k] - eye[k]) / dir[k]);
                near = near.max(t0.min(t1));
                far = far.min(t0.max(t1));
            }
        }
        if hit && near <= far && best.is_none_or(|(d, _)| near < d) {
            best = Some((near, id));
        }
    }
    best.map(|(_, id)| id)
}

/// Inventories asked for with a chest, to open on the next tick: a menu can't be opened while the
/// server is in the middle of the player's own click (single player is an integrated server too).
static PENDING: Mutex<Vec<(u128, u32)>> = Mutex::new(Vec::new());

/// The bauble items in a doll's grid, by name.
fn worn_names(baubles: &[Option<ItemStack>]) -> Vec<std::sync::Arc<str>> {
    baubles.iter().flatten().filter(|s| goals::is_bauble(&s.item)).map(|s| s.item.clone()).collect()
}

/// The four rows of the bauble grid in a chest screen: its baubles in the middle four columns, panes
/// around them that can't be taken.
fn bauble_block(baubles: &[Option<ItemStack>]) -> Vec<Option<ItemStack>> {
    let mut pane = ItemStack::new("minecraft:gray_stained_glass_pane", 1);
    pane.extra = Some(Box::new(StackData { custom_name: Some(Tag::String(" ".into())), ..StackData::default() }));
    (0..goals::BAUBLE_CELLS)
        .map(|i| match goals::bauble_cell(i, 0) {
            Some(Some(n)) => baubles.get(n).cloned().flatten(),
            _ => Some(pane.clone()),
        })
        .collect()
}

/// The baubles out of a closed screen's bauble rows (the panes left behind).
fn baubles_from(cells: &mut [Option<ItemStack>]) -> Vec<Option<ItemStack>> {
    let mut b = vec![None; goals::BAUBLE_SLOTS];
    for (i, cell) in cells.iter_mut().enumerate() {
        match goals::bauble_cell(i, 0) {
            Some(Some(n)) => b[n] = cell.take(),
            _ => *cell = None,
        }
    }
    b
}

/// Bauble screens asked for (a doll right-clicked with a bauble), to open next tick; those open, by
/// player; and the grids that closed, to save.
static PENDING_BAUBLES: Mutex<Vec<(u128, u32)>> = Mutex::new(Vec::new());
/// Players whose open doll screen has the bauble grid under the inventory: which of the doll's
/// slots it shows, from where.
static GRID_OPEN: LazyLock<Mutex<HashMap<u128, (usize, usize)>>> = LazyLock::new(Default::default);
/// Where each player has scrolled a long doll inventory to, in rows; and wheel turns to act on.
static SCROLL: LazyLock<Mutex<HashMap<u128, usize>>> = LazyLock::new(Default::default);
static SCROLLED: Mutex<Vec<(u128, i32)>> = Mutex::new(Vec::new());
/// Scrolled screens to open again, with how many ticks they have waited for the old one to close.
static REOPEN: Mutex<Vec<(u128, u32, u8)>> = Mutex::new(Vec::new());
/// Doll screens saved when scrolled away from, by player and window: their close gives nothing back.
static SAVED_WINDOWS: Mutex<Vec<(u128, i32)>> = Mutex::new(Vec::new());
static BAUBLE_OPEN: LazyLock<Mutex<HashMap<u128, (u32, i32)>>> = LazyLock::new(Default::default);
static BAUBLES_CLOSED: Mutex<Vec<(u32, Vec<Option<ItemStack>>)>> = Mutex::new(Vec::new());


/// Panels of players to choose from, asked for and open (the doll, the window, each slot's player).
static PENDING_GUIGNOL: Mutex<Vec<(u128, u32)>> = Mutex::new(Vec::new());
static PICKING: LazyLock<Mutex<HashMap<u128, (u32, i32, Vec<(u128, String)>)>>> = LazyLock::new(Default::default);
static MISSIONS: LazyLock<Mutex<HashMap<u32, Mission>>> = LazyLock::new(Default::default);

#[derive(Clone, Copy, Debug, PartialEq)]
enum Phase {
    /// Invisible, a few blocks behind them, following.
    Stalk,
    /// Running at them.
    Charge,
    /// Gone off: a moment on, once the blast is over, the skeletons come and it goes home.
    Done,
}

#[derive(Clone, Copy, Debug)]
struct Mission {
    owner: u128,
    target: u128,
    ticks: u32,
    phase: Phase,
}

const GUIGNOL_SOUND: &str = "alice_dolls:guignol_visit";
const SOUND_HOSTILE: usize = 5;

/// How long it stalks, how long it may take to reach them, how near it goes off, and how long after
/// it goes home.
const STALK_TICKS: u32 = 60;
const CHARGE_TICKS: u32 = 60;
const CHARGE_STEP: f64 = 0.7;
const BURST_REACH: f64 = 1.6;
/// Ticks after it goes off before the skeletons come and it is home: the blast comes on the doll's
/// next tick, so they never feel it.
const HOME_AFTER: u32 = 2;

/// A skeleton in diamond armour, with a bow or an iron sword.
fn armoured_skeleton(weapon: &str) -> Compound {
    let item = |id: &str| Tag::Compound([("id".into(), Tag::String(id.into())), ("count".into(), Tag::Int(1))].into());
    let equipment: Compound = [
        ("head".into(), item("minecraft:diamond_helmet")),
        ("chest".into(), item("minecraft:diamond_chestplate")),
        ("legs".into(), item("minecraft:diamond_leggings")),
        ("feet".into(), item("minecraft:diamond_boots")),
        ("mainhand".into(), item(weapon)),
    ]
    .into();
    [("equipment".into(), Tag::Compound(equipment))].into()
}

impl Mod {
    /// The panel: a head for each other player on the server or the LAN.
    fn open_guignol_panel(&self, host: &mut dyn Host, owner: u128, doll: u32) {
        // Every player but spectators, its owner first (marked).
        let mut players: Vec<(u128, String)> = host
            .server_players()
            .into_iter()
            .filter(|p| p.game_mode != 3)
            .map(|p| if p.uuid == owner { (p.uuid, format!("{} (you)", p.name)) } else { (p.uuid, p.name) })
            .collect();
        players.sort_by_key(|(u, _)| *u != owner);
        if players.is_empty() {
            host.action_bar(owner, Text::literal("Grand Guignol: there's no one here to visit"));
            return;
        }
        let shown: Vec<Option<ItemStack>> = players
            .iter()
            .take(54)
            .map(|(_, name)| {
                let mut head = ItemStack::new("minecraft:player_head", 1);
                head.extra = Some(Box::new(StackData { custom_name: Some(Tag::String(name.as_str().into())), ..StackData::default() }));
                Some(head)
            })
            .collect();
        let rows = shown.len().div_ceil(9).clamp(1, 6) as u8;
        let Some(window) = host.open_menu(owner, rows, Text::literal("Grand Guignol: who shall I visit?"), shown) else { return };
        PICKING.lock().ok().map(|mut p| p.insert(owner, (doll, window, players)));
    }

    fn mission_tick(&self, host: &mut dyn Host, doll: u32) {
        let Some(mut mission) = MISSIONS.lock().ok().and_then(|m| m.get(&doll).copied()) else { return };
        let players = host.server_players();
        let Some(entity) = host.entity(doll) else { return };
        let (Some(at), Some(level)) = (host.entity_position(entity), host.entity_level(entity)) else { return };
        let dimension = level.dimension().to_owned();
        let target = players.iter().find(|p| p.uuid == mission.target && p.dimension == dimension);
        mission.ticks += 1;
        let done = |host: &mut dyn Host, doll: u32, owner: u128, players: &[NativeServerPlayer]| {
            MISSIONS.lock().ok().map(|mut m| m.remove(&doll));
            goals::set_mission(doll, false);
            // Straight back to its owner, or, with them in another dimension, to its anchor.
            let to_owner = players
                .iter()
                .find(|p| p.uuid == owner && p.dimension == dimension)
                .map(|o| [o.position[0] + 1.0, o.position[1], o.position[2] + 1.0]);
            let to_anchor = DOLL
                .get(host, Holder::Entity(doll))
                .ok()
                .and_then(|d| d.anchor)
                .map(|a| [f64::from(a[0]) + 0.5, f64::from(a[1]) + 1.0, f64::from(a[2]) + 0.5]);
            if let Some(home) = to_owner.or(to_anchor) {
                host.teleport_mob(&dimension, doll, home);
            }
        };
        let Some(t) = target else {
            // Gone, or in another dimension: it comes home.
            if mission.phase != Phase::Done {
                host.action_bar(mission.owner, Text::literal("Grand Guignol couldn't reach them, and came back"));
            }
            done(host, doll, mission.owner, &players);
            return;
        };
        let to = t.position;
        let (dx, dz) = (to[0] - at[0], to[2] - at[2]);
        let near = (dx * dx + dz * dz + (to[1] - at[1]).powi(2)).sqrt();
        match mission.phase {
            Phase::Stalk => {
                // Four blocks behind them, where they aren't looking.
                if mission.ticks % 5 == 1 {
                    let yaw = f64::from(t.yaw).to_radians();
                    let behind = [to[0] + yaw.sin() * 4.0, to[1], to[2] - yaw.cos() * 4.0];
                    host.teleport_mob(&dimension, doll, behind);
                }
                if mission.ticks >= STALK_TICKS {
                    mission.phase = Phase::Charge;
                    mission.ticks = 0;
                    host.play_local_sound("minecraft:entity.creeper.primed", at, 1.0, 0.5, 0);
                }
            }
            Phase::Charge => {
                if near <= BURST_REACH || mission.ticks >= CHARGE_TICKS {
                    goals::burst(doll);
                    // Its own sound for everyone near, and a word for the one it visited.
                    host.level_play_sound(level, GUIGNOL_SOUND, at, SOUND_HOSTILE, 3.0, 1.0);
                    host.tell(t.uuid, Text::literal("You have been visited by the Grand Guignol"));
                    mission.phase = Phase::Done;
                    mission.ticks = 0;
                } else {
                    let step = CHARGE_STEP.min(near);
                    let next = [at[0] + (to[0] - at[0]) / near * step, to[1], at[2] + (to[2] - at[2]) / near * step];
                    host.teleport_mob(&dimension, doll, next);
                }
            }
            Phase::Done => {
                if mission.ticks >= HOME_AFTER {
                    // The blast is over: now the skeletons, round whoever it visited. They spare its
                    // owner and their dolls.
                    for i in 0..6 {
                        let a = f64::from(i) * std::f64::consts::TAU / 6.0;
                        let spot = [to[0] + a.cos() * 2.5, to[1], to[2] + a.sin() * 2.5];
                        let weapon = if i % 2 == 0 { "minecraft:iron_sword" } else { "minecraft:bow" };
                        if let Some(id) = host.summon(&dimension, "minecraft:skeleton", spot, armoured_skeleton(weapon)) {
                            goals::guard(id, mission.owner);
                        }
                    }
                    done(host, doll, mission.owner, &players);
                    let data = DOLL.get(host, Holder::Entity(doll)).unwrap_or_default();
                    say(host, doll, &data, 1, "I'm back");
                    return;
                }
            }
        }
        MISSIONS.lock().ok().map(|mut m| m.insert(doll, mission));
    }
}

/// Aya's photo being taken, and the frames left drawn from her eyes: the screenshot is of the next
/// frame, and a few more cover a frame drawn late.
static AYA_POV: Mutex<Option<(goals::Photo, i32)>> = Mutex::new(None);
/// The HUD mode to go back to once her photo is taken.
static AYA_HUD: Mutex<Option<HudMode>> = Mutex::new(None);
const AYA_POV_FRAMES: i32 = 4;

/// The doll inventory screen open on this client: the doll's name (its title), its slot count, and
/// which slots are empty, from the server's tick (single player shares the process).
static MENU_VIEW: Mutex<Option<MenuView>> = Mutex::new(None);

/// The doll screen open: its title, the doll's slot count, which of them the screen shows from
/// where, and which of those are empty.
#[derive(Clone, Debug)]
struct MenuView {
    title: String,
    total: usize,
    offset: usize,
    view: usize,
    empty: Vec<bool>,
}
/// Whether this screen's outlines were logged yet (once a screen, to the debug log).
static OUTLINES_LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

// ---- The doll anchor ----

const ANCHOR: &str = "alice_dolls:doll_anchor";
const ANCHORS_FILE: &str = "doll_anchors.txt";
/// Every doll anchor: its dimension, where it is, and whose.
static ANCHORS: Mutex<Vec<(String, [i32; 3], u128)>> = Mutex::new(Vec::new());
static ANCHORS_LOADED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static ANCHORS_CHANGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static ANCHOR_TICKS: LazyLock<Mutex<HashMap<u32, u32>>> = LazyLock::new(Default::default);
/// How far a doll looks for its owner's anchor.
const ANCHOR_RANGE: f64 = 256.0;
/// An anchor's radius: how near it a doll must be, when placed or set to patrol, to be bound to it,
/// and how far a patrolling doll goes from it before heading back. Sneak and right-click an anchor
/// to raise it by ten, up to a hundred; past that it starts again at thirty.
const ANCHOR_RADIUS: u32 = 30;
const ANCHOR_RADIUS_MAX: u32 = 100;
static RADII: LazyLock<Mutex<HashMap<[i32; 3], u32>>> = LazyLock::new(Default::default);

fn radius_of(p: [i32; 3]) -> u32 {
    RADII.lock().ok().and_then(|r| r.get(&p).copied()).unwrap_or(ANCHOR_RADIUS)
}

/// The owner's nearest anchor whose radius `at` is within, in `dimension`.
fn nearest_anchor(dimension: &str, owner: u128, at: [f64; 3]) -> Option<[i32; 3]> {
    let d2 = |p: [i32; 3]| (f64::from(p[0]) + 0.5 - at[0]).powi(2) + (f64::from(p[1]) - at[1]).powi(2) + (f64::from(p[2]) + 0.5 - at[2]).powi(2);
    ANCHORS
        .lock()
        .ok()?
        .iter()
        .filter(|(d, p, o)| d == dimension && *o == owner && d2(*p) <= f64::from(radius_of(*p)).powi(2))
        .map(|(_, p, _)| *p)
        .min_by(|a, b| d2(*a).total_cmp(&d2(*b)))
}

/// Within this of its anchor, a doll empties what it carries into a chest beside or on it.
const DEPOSIT_RANGE_SQR: f64 = 16.0;

/// Where a chest beside an anchor, or on it, may be.
const CHEST_SIDES: [([i32; 3], Direction); 5] = [
    ([0, 1, 0], Direction::Down),
    ([1, 0, 0], Direction::West),
    ([-1, 0, 0], Direction::East),
    ([0, 0, 1], Direction::North),
    ([0, 0, -1], Direction::South),
];

/// Whether a slot of a doll's inventory is one it keeps when it empties itself: its armour and shield,
/// and a Russia doll's torches.
fn kept_slot(n: usize, i: usize, item: &str) -> bool {
    n > goals::ARMOUR_SLOTS && (i >= goals::shield_slot(n)) || item == "minecraft:torch"
}

impl Mod {
    /// Once a second for each doll: which anchor it keeps to, and emptying itself into the chest
    /// there when it is close, or from anywhere when it wears the Gap and is full.
    fn anchor_work(&self, host: &mut dyn Host, id: u32, data: &mut DollData) {
        let Some(entity) = host.entity(id) else { return };
        let (Some(at), Some(level)) = (host.entity_position(entity), host.entity_level(entity)) else { return };
        let dimension = level.dimension().to_owned();
        let d2 = |p: [i32; 3]| (f64::from(p[0]) + 0.5 - at[0]).powi(2) + (f64::from(p[1]) - at[1]).powi(2) + (f64::from(p[2]) + 0.5 - at[2]).powi(2);
        // Its own anchor, the one it was bound to: still there, in this dimension, and on.
        let listed = |p: [i32; 3]| ANCHORS.lock().is_ok_and(|a| a.iter().any(|(d, q, _)| *d == dimension && *q == p));
        let mut anchor = data.anchor.filter(|&p| listed(p));
        if let Some(p) = anchor
            && host.level_block(level, p).is_some_and(|b| !b.starts_with(ANCHOR))
        {
            // Gone (broken some way the loot hook didn't see): forgotten.
            if let Ok(mut a) = ANCHORS.lock() {
                a.retain(|x| x.1 != p);
            }
            ANCHORS_CHANGED.store(true, std::sync::atomic::Ordering::Relaxed);
            data.anchor = None;
            anchor = None;
        }
        // A lever or any redstone signal on it switches it off.
        let anchor = anchor.filter(|&p| host.neighbor_signal(&dimension, p) == 0 && d2(p) <= ANCHOR_RANGE * ANCHOR_RANGE);
        let patrol = data.enabled && data.mode == goals::PATROL;
        goals::set_anchor(id, anchor.filter(|_| patrol).map(|p| (p, f64::from(radius_of(p)))));
        let Some(anchor) = anchor else { return };
        let mut items = goals::inventory(id).unwrap_or_else(|| data.items.clone());
        let n = items.len();
        let full = items.iter().enumerate().all(|(i, s)| s.is_some() || kept_slot(n, i, ""));
        let gap = goals::wears(id, "alice_dolls:wireless_io") && full;
        if !(patrol && d2(anchor) <= DEPOSIT_RANGE_SQR) && !gap {
            return;
        }
        if goals::chest_open() == Some(id) {
            return;
        }
        let chests: Vec<([i32; 3], Direction)> = CHEST_SIDES
            .iter()
            .map(|(o, face)| ([anchor[0] + o[0], anchor[1] + o[1], anchor[2] + o[2]], *face))
            .filter(|(p, _)| {
                host.level_block(level, *p)
                    .is_some_and(|b| (b.contains("chest") || b.contains("barrel")) && !b.contains("ender_chest"))
            })
            .collect();
        if chests.is_empty() {
            return;
        }
        let mut moved = 0;
        for (i, slot) in items.iter_mut().enumerate() {
            let Some(stack) = slot.take() else { continue };
            if kept_slot(n, i, &stack.item) {
                *slot = Some(stack);
                continue;
            }
            let mut rest = Some(stack);
            for (p, face) in &chests {
                let Some(s) = rest.take() else { break };
                let count = s.count;
                rest = host.insert_item(*p, *face, s);
                if rest.as_ref().is_none_or(|r| r.count < count) {
                    moved += 1;
                }
            }
            *slot = rest;
        }
        if moved == 0 {
            return;
        }
        say(host, id, data, 3, if gap { "Sent my things home through the Gap" } else { "Put my things in the chest" });
        goals::put_inventory(id, items.clone());
        data.items = items;
        if let Some(doll) = doll_of(host, id) {
            remember(id, doll, data);
        }
        let _ = DOLL.set(host, Holder::Entity(id), data.clone());
    }
}

/// Where the switch sits in the row under a Horai's slots.
const SWITCH_AT: usize = 4;

/// The dolls whose inventory screen has the keep-or-void switch: the ones that quarry and hunt.
fn has_rock_switch(name: &str) -> bool {
    name == goals::DIAMOND_HORAI || name == goals::NETHERITE_HORAI
}

/// The row under a Horai's slots: the switch in the middle, glass panes around it.
fn switch_row(keep: bool) -> Vec<Option<ItemStack>> {
    let named = |item: &str, text: &str| {
        let mut s = ItemStack::new(item, 1);
        s.extra = Some(Box::new(StackData { custom_name: Some(Tag::String(text.into())), ..StackData::default() }));
        Some(s)
    };
    let mut row = vec![named("minecraft:gray_stained_glass_pane", " "); 9];
    row[SWITCH_AT] = if keep {
        named("minecraft:cobblestone", "Keeping stone and earth: click to void it")
    } else {
        named("minecraft:lava_bucket", "Voiding stone and earth: click to keep it")
    };
    row
}

/// Where the placed blocks are kept, in the mod's config folder.
const PLACED_FILE: &str = "player_placed_blocks.txt";
static PLACED_LOADED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static PLACED_TICKS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
/// For the shield's blocks.
static SHIELD_ROLL: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0x9E37_79B9);

/// The doll a Moscow doll was just set to follow, and when: it glows, with a marker over it.
static HIGHLIGHT: Mutex<Option<(u32, Instant)>> = Mutex::new(None);
const HIGHLIGHT_FOR: Duration = Duration::from_secs(4);

fn highlight(host: &mut dyn Host, id: u32) {
    let old = HIGHLIGHT.lock().ok().and_then(|mut h| h.replace((id, Instant::now())));
    if let Some((old, _)) = old
        && old != id
    {
        set_glowing(host, old, false);
    }
    set_glowing(host, id, true);
}

fn set_glowing(host: &mut dyn Host, id: u32, on: bool) {
    if let Some(entity) = host.entity(id) {
        host.entity_merge_nbt(entity, [("Glowing".into(), Tag::Byte(i8::from(on)))].into());
    }
}

/// Stacks of the same item put together, up to `max` each: two 64s in a Moscow doll become one 128.
fn compact(items: &mut [Option<ItemStack>], max: u32) {
    for i in 0..items.len() {
        for j in (i + 1)..items.len() {
            let (a, b) = items.split_at_mut(j);
            if let (Some(x), Some(y)) = (a[i].as_mut(), b[0].as_mut())
                && same_kind(x, y)
                && x.count < max
            {
                let add = y.count.min(max - x.count);
                x.count += add;
                y.count -= add;
                if y.count == 0 {
                    b[0] = None;
                }
            }
        }
    }
}

/// The Moscow doll each player is binding with a lead.
static BINDING: LazyLock<Mutex<HashMap<u128, u32>>> = LazyLock::new(Default::default);
/// Each doll's entity id by its UUID, for a Moscow doll to find hers.
static UUIDS: LazyLock<Mutex<HashMap<u128, u32>>> = LazyLock::new(Default::default);
/// Ticks of each Moscow doll's, between looks at her doll's inventory.
static MOSCOW_TICKS: LazyLock<Mutex<HashMap<u32, u32>>> = LazyLock::new(Default::default);

impl Mod {
    /// Gives doll `id` `points` of favour: on a new level it says so, heals to its new health, and
    /// a Horai or Goliath three levels into its form grows into the next. The caller saves `data`.
    fn add_favour(&self, host: &mut dyn Host, id: u32, data: &mut DollData, points: u32) {
        let before = favour_level(data.favour).0;
        data.favour = data.favour.saturating_add(points);
        let after = favour_level(data.favour).0;
        goals::set_level(id, after);
        if after <= before {
            return;
        }
        if let Some(at) = host.entity(id).and_then(|e| host.entity_position(e)) {
            host.play_local_sound("minecraft:entity.player.levelup", at, 0.8, 1.2, 0);
        }
        say(host, id, data, 1, &format!("Love level {after}!"));
        let Some(doll) = doll_of(host, id) else { return };
        if after.saturating_sub(data.form_level) < LEVELS_PER_FORM {
            remember(id, doll, data);
            return;
        }
        let Some(next) = next_form(doll.name).and_then(doll_named) else {
            remember(id, doll, data);
            return;
        };
        self.evolve(host, id, data, next, after);
    }

    /// Doll `id` becomes `next`: its name (which is its kind), its slots and its health.
    fn evolve(&self, host: &mut dyn Host, id: u32, data: &mut DollData, next: &'static Doll, level: u32) {
        let Some(entity) = host.entity(id) else { return };
        host.entity_merge_nbt(entity, [("CustomName".into(), Tag::String(next.name.into()))].into());
        data.form_level = level;
        if data.items.len() < next.slots {
            let old = data.items.len();
            data.items.resize(next.slots, None);
            if old > goals::ARMOUR_SLOTS + 1 {
                for k in 0..=goals::ARMOUR_SLOTS {
                    data.items.swap(old - 1 - k, next.slots - 1 - k);
                }
            }
            goals::put_inventory(id, data.items.clone());
        }
        remember(id, next, data);
        tell_state(id, data);
        if let Some(at) = host.entity_position(entity) {
            host.play_local_sound("minecraft:ui.toast.challenge_complete", at, 1.0, 1.0, 0);
        }
        say(host, id, data, 1, &format!("I've become a {}!", next.name));
    }

    fn interact_doll(&self, host: &mut dyn Host, id: u32, held: Option<&str>, stack: Option<&ItemStack>) -> Flow {
        let holder = Holder::Entity(id);
        if !DOLL.has(host, holder).unwrap_or(false) {
            return Flow::Continue;
        }
        let Some(player) = host.acting_player() else {
            return Flow::Continue;
        };
        if !fresh_click(player.uuid) {
            return Flow::Cancel;
        }
        let mut data = DOLL.get(host, holder).unwrap_or_default();
        let name = doll_of(host, id).map_or("Doll", |d| d.name);
        if data.owner != player.uuid {
            if held != Some(DOLL_CORE) {
                let text = format!("{name} : You are not my owner!");
                host.action_bar(player.uuid, Text::literal(text));
                return Flow::Cancel;
            }
            // A Doll Core takes the doll over, and it starts again from Standby.
            if player.game_mode != 1 {
                host.take_items(DOLL_CORE, 1);
            }
            data.owner = player.uuid;
            data.mode = goals::STANDBY;
            if let Some(doll) = doll_of(host, id) {
                remember(id, doll, &data);
            }
            tell_state(id, &data);
            if let Some(at) = host.entity(id).and_then(|e| host.entity_position(e)) {
                host.play_local_sound("minecraft:item.flintandsteel.use", at, 1.0, 1.0, 0);
            }
            say(host, id, &data, 2, "Activation successful!");
            let _ = DOLL.set(host, holder, data);
            return Flow::Cancel;
        }
        if held == Some("minecraft:lead") && self.bind(host, id, &mut data, &player) {
            tell_state(id, &data);
            let _ = DOLL.set(host, holder, data);
            return Flow::Cancel;
        }
        if goals::player_sneaking() && held == Some("minecraft:crafting_table") && (name == goals::MOSCOW || name == goals::RUSSIA) {
            self.next_bound(host, id, &mut data, &player);
            tell_state(id, &data);
            let _ = DOLL.set(host, holder, data);
            return Flow::Cancel;
        }
        if goals::player_sneaking() {
            let seeker = doll_of(host, id).is_some_and(|d| d.name == goals::DIAMOND_HORAI || d.name == goals::NETHERITE_HORAI);
            let glass = held == Some("minecraft:red_stained_glass");
            let to = if data.mode == goals::SECRET && glass {
                Some(goals::SECRET_ALL)
            } else if data.mode == goals::SECRET || data.mode == goals::SECRET_ALL {
                Some(goals::STANDBY)
            } else if seeker && glass {
                Some(goals::SECRET)
            } else {
                None
            };
            if let Some(mode) = to {
                data.mode = mode;
                say(host, id, &data, 1, mode_name(mode));
                click(host, id);
                tell_state(id, &data);
                let _ = DOLL.set(host, holder, data);
                return Flow::Cancel;
            }
        }
        if held.is_none() && goals::player_sneaking() {
            // Sneak and right-click with an empty hand: the doll comes back as its item, with what
            // it carries.
            if let Some(doll) = doll_of(host, id) {
                if data.xp > 0 {
                    host.give_experience(player.uuid, data.xp);
                }
                forget(id);
                host.remove_mob(&player.dimension, id);
                let items = goals::inventory(id).unwrap_or(data.items);
                let k = Known {
                    doll,
                    owner: player.uuid,
                    items,
                    baubles: data.baubles.clone(),
                    favour: data.favour,
                    form_level: data.form_level,
                };
                give_to_owner(host, player.uuid, belongings(&k));
            }
            return Flow::Cancel;
        }
        match held {
            // The original let a name tag rename a doll; here the name is what makes it one.
            Some("minecraft:name_tag") => {}
            // A Point or Power item, dropped by fairies and other Touhou mobs: favour. A right-click
            // gives the stack; sneak to give a little, one at a time.
            Some(item) if point_favour(item).is_some() => {
                let worth = point_favour(item).unwrap_or(POINT_FAVOUR);
                let held_count = stack.map_or(1, |s| s.count.max(1));
                let n = if goals::player_sneaking() { 1 } else { held_count };
                let given = if player.game_mode == 1 { n } else { host.take_items(item, n) };
                if given > 0 {
                    self.add_favour(host, id, &mut data, given * worth);
                    let (level, into, need) = favour_level(data.favour);
                    let text = format!("{name} : love level {level} ({into}/{need})");
                    host.action_bar(player.uuid, Text::literal(text));
                }
            }
            // Wakasagi and a fishing rod: she fishes with it from now on (its enchantments too), and
            // hands back the rod she had.
            Some("minecraft:fishing_rod") if name == goals::WAKASAGI => {
                if let Some(rod) = stack {
                    self.give_rod(host, id, &mut data, &player, rod);
                }
            }
            // A glass bottle: the experience it picked up comes out, all of it.
            Some("minecraft:glass_bottle") => {
                if data.xp == 0 {
                    say(host, id, &data, 1, "I have no experience to give");
                } else {
                    let xp = std::mem::take(&mut data.xp);
                    host.give_experience(player.uuid, xp);
                    say(host, id, &data, 1, &format!("Here are {xp} experience points"));
                }
            }
            Some(DOLL_CORE) => {
                data.enabled = !data.enabled;
                let text = if data.enabled { "Work AI ON" } else { "Work AI OFF" };
                say(host, id, &data, 1, text);
                click(host, id);
            }
            Some("minecraft:book") => {
                data.chat_level = (data.chat_level + 1) % 4;
                let text = format!("{name} : setChatLevel({})", data.chat_level);
                host.tell(data.owner, Text::literal(text));
                click(host, id);
            }
            Some("minecraft:chest") => {
                if let Ok(mut pending) = PENDING.lock() {
                    pending.push((player.uuid, id));
                }
                return Flow::Cancel;
            }
            // A doll puts on armour or a shield it is handed, taking off what it had there.
            Some(item) if goals::armour_part(item).is_some() || item == "minecraft:shield" => {
                self.equip(host, id, &mut data, &player, stack);
            }
            Some("minecraft:netherite_ingot") if name == goals::GRAND_GUIGNOL => {
                if goals::on_mission(id) {
                    say(host, id, &data, 1, "I'm busy");
                    return Flow::Cancel;
                }
                if player.game_mode != 1 && host.take_items("minecraft:netherite_ingot", 1) != 1 {
                    return Flow::Cancel;
                }
                if let Ok(mut p) = PENDING_GUIGNOL.lock() {
                    p.push((player.uuid, id));
                }
                return Flow::Cancel;
            }
            // A bauble opens its bauble screen.
            Some(item) if goals::is_bauble(item) => {
                if let Ok(mut p) = PENDING_BAUBLES.lock() {
                    p.push((player.uuid, id));
                }
                return Flow::Cancel;
            }
            _ => {
                data.mode = next_mode_of(name, data.mode);
                if data.mode == goals::PATROL
                    && let Some(at) = host.entity(id).and_then(|e| host.entity_position(e))
                {
                    data.anchor = nearest_anchor(&player.dimension, player.uuid, at);
                    if data.anchor.is_some() {
                        say(host, id, &data, 2, "Anchored");
                    }
                }
                say(host, id, &data, 1, mode_name(data.mode));
                click(host, id);
            }
        }
        tell_state(id, &data);
        let _ = DOLL.set(host, holder, data);
        Flow::Cancel
    }

    /// A doll puts on the armour piece `stack` from its owner's hand, in its own slot of the last
    /// four; what was in that slot goes back to them.
    fn equip(&self, host: &mut dyn Host, id: u32, data: &mut DollData, player: &NativeServerPlayer, stack: Option<&ItemStack>) {
        let Some(stack) = stack else { return };
        let part = goals::armour_part(&stack.item);
        if part.is_none() && &*stack.item != "minecraft:shield" {
            return;
        }
        if goals::chest_open() == Some(id) {
            return;
        }
        let mut items = goals::inventory(id).unwrap_or_else(|| data.items.clone());
        if items.len() < goals::ARMOUR_SLOTS {
            return;
        }
        if player.game_mode != 1 && host.take_items(&stack.item, 1) != 1 {
            return;
        }
        let mut piece = stack.clone();
        piece.count = 1;
        // Into that part's own slot: whatever was there comes out.
        let at = match part {
            Some(part) => goals::armour_slot(items.len(), part),
            None => goals::shield_slot(items.len()),
        };
        let old = items[at].replace(piece);
        if let Some(old) = old {
            for left in give_to_owner(host, player.uuid, vec![old]) {
                if let Some(slot) = items.iter_mut().find(|s| s.is_none()) {
                    *slot = Some(left);
                }
            }
        }
        let short = stack.item.rsplit(':').next().unwrap_or("armour").replace('_', " ");
        say(host, id, data, 1, &format!("Wearing the {short}"));
        if let Some(at) = host.entity(id).and_then(|e| host.entity_position(e)) {
            host.play_local_sound("minecraft:item.armor.equip_iron", at, 1.0, 1.0, 0);
        }
        goals::put_inventory(id, items.clone());
        data.items = items;
        if let Some(doll) = doll_of(host, id) {
            remember(id, doll, data);
        }
    }

    /// Wakasagi takes the rod `rod` from `player`'s hand into her inventory, where she fishes with
    /// the first rod she has: the rod she had before comes back to them.
    fn give_rod(&self, host: &mut dyn Host, id: u32, data: &mut DollData, player: &NativeServerPlayer, rod: &ItemStack) {
        if goals::chest_open() == Some(id) {
            return;
        }
        let mut items = goals::inventory(id).unwrap_or_else(|| data.items.clone());
        let old_at = items.iter().position(|s| s.as_ref().is_some_and(|s| &*s.item == "minecraft:fishing_rod"));
        let at = match old_at.or_else(|| items.iter().position(Option::is_none)) {
            Some(at) => at,
            None => {
                say(host, id, data, 1, "My inventory is full");
                return;
            }
        };
        if player.game_mode != 1 && host.take_items("minecraft:fishing_rod", 1) != 1 {
            return;
        }
        let mut new = rod.clone();
        new.count = 1;
        if let Some(old) = items[at].replace(new) {
            give_to_owner(host, player.uuid, vec![old]);
        }
        say(host, id, data, 1, "I'll fish with this one");
        if let Some(at) = host.entity(id).and_then(|e| host.entity_position(e)) {
            host.play_local_sound("minecraft:entity.fishing_bobber.throw", at, 0.8, 1.0, 0);
        }
        goals::put_inventory(id, items.clone());
        data.items = items;
        if let Some(doll) = doll_of(host, id) {
            remember(id, doll, data);
        }
    }

    /// A lead binds a Moscow doll to another doll: sneak and right-click Moscow with it, then
    /// right-click the other doll. Sneak and right-click a bound Moscow with it to set her free.
    /// Whether the click was this.
    fn bind(&self, host: &mut dyn Host, id: u32, data: &mut DollData, player: &NativeServerPlayer) -> bool {
        let name = doll_of(host, id).map_or("", |d| d.name);
        if (name == goals::MOSCOW || name == goals::RUSSIA) && goals::player_sneaking() {
            if data.bound != 0 {
                data.bound = 0;
                goals::set_bound(id, None);
                say(host, id, data, 1, "Unbound: back to looking after you");
            } else {
                if let Ok(mut b) = BINDING.lock() {
                    b.insert(player.uuid, id);
                }
                say(host, id, data, 1, "Right-click one of your dolls with the lead to bind me to it");
            }
            click(host, id);
            return true;
        }
        let Some(moscow) = BINDING.lock().ok().and_then(|mut b| b.remove(&player.uuid)) else { return false };
        if moscow == id {
            return true;
        }
        let Some(uuid) = host.entity(id).and_then(|e| host.entity_uuid(e)) else { return true };
        let holder = Holder::Entity(moscow);
        if !DOLL.has(host, holder).unwrap_or(false) {
            return true;
        }
        let mut m = DOLL.get(host, holder).unwrap_or_default();
        if m.owner != player.uuid {
            return true;
        }
        m.bound = uuid;
        goals::set_bound(moscow, Some(id));
        say(host, moscow, &m, 1, &format!("Bound to {name}: I'll follow it and carry what it can't"));
        let _ = DOLL.set(host, holder, m);
        click(host, id);
        true
    }

    /// Sneak and right-click a Moscow doll with a crafting table: she moves on to the next of her
    /// owner's placed dolls, which glows for a few seconds, and follows it from then on.
    fn next_bound(&self, host: &mut dyn Host, id: u32, data: &mut DollData, player: &NativeServerPlayer) {
        let mut ids: Vec<u32> = KNOWN
            .lock()
            .map(|k| {
                k.iter()
                    .filter(|(other, k)| **other != id && k.owner == player.uuid && k.doll.name != goals::MOSCOW)
                    .map(|(other, _)| *other)
                    .collect()
            })
            .unwrap_or_default();
        ids.retain(|&other| host.entity(other).is_some());
        ids.sort_unstable();
        if ids.is_empty() {
            say(host, id, data, 1, "You have no other dolls out for me to follow");
            return;
        }
        let now = UUIDS.lock().ok().and_then(|u| u.get(&data.bound).copied());
        let at = now.and_then(|n| ids.iter().position(|&o| o == n)).map_or(0, |i| (i + 1) % ids.len());
        let next = ids[at];
        let Some(uuid) = host.entity(next).and_then(|e| host.entity_uuid(e)) else { return };
        data.bound = uuid;
        goals::set_bound(id, Some(next));
        highlight(host, next);
        let name = doll_of(host, next).map_or("Doll", |d| d.name);
        say(host, id, data, 1, &format!("Following {name} ({} of {})", at + 1, ids.len()));
        click(host, id);
    }

    /// The doll bell: every one of the ringer's dolls teleports to them and stands by.
    fn ring_bell(&self, host: &mut dyn Host) -> Flow {
        let Some(player) = host.acting_player() else { return Flow::Continue };
        if !fresh_click(player.uuid) {
            return Flow::Cancel;
        }
        let ids: Vec<u32> = KNOWN
            .lock()
            .map(|k| k.iter().filter(|(_, k)| k.owner == player.uuid).map(|(id, _)| *id).collect())
            .unwrap_or_default();
        let mut came = 0;
        for (n, id) in ids.into_iter().enumerate() {
            let a = n as f64 * 2.4;
            let at = [player.position[0] + a.cos() * 0.8, player.position[1], player.position[2] + a.sin() * 0.8];
            if !host.teleport_mob(&player.dimension, id, at) {
                continue;
            }
            came += 1;
            let holder = Holder::Entity(id);
            if DOLL.has(host, holder).unwrap_or(false) {
                let mut data = DOLL.get(host, holder).unwrap_or_default();
                data.mode = goals::STANDBY;
                tell_state(id, &data);
                let _ = DOLL.set(host, holder, data);
            }
        }
        host.play_local_sound("minecraft:block.bell.use", player.position, 1.0, 1.4, 0);
        let text = match came {
            0 => "No dolls answered the bell".to_owned(),
            1 => "1 doll came back, on standby".to_owned(),
            n => format!("{n} dolls came back, on standby"),
        };
        host.action_bar(player.uuid, Text::literal(text));
        Flow::Cancel
    }

    /// A Moscow doll empties the doll she is bound to when it is full, into her own stacks of 128:
    /// all but what it wears (a Goliath's armour) and its torches.
    fn take_from_bound(&self, host: &mut dyn Host, id: u32, data: &mut DollData) {
        let Some(target) = UUIDS.lock().ok().and_then(|u| u.get(&data.bound).copied()) else { return };
        if goals::chest_open() == Some(id) || goals::chest_open() == Some(target) {
            return;
        }
        let (Some(a), Some(b)) = (
            host.entity(id).and_then(|e| host.entity_position(e)),
            host.entity(target).and_then(|e| host.entity_position(e)),
        ) else {
            return;
        };
        if (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2) > 64.0 {
            return;
        }
        let Some(mut theirs) = goals::inventory(target) else { return };
        if theirs.iter().any(Option::is_none) {
            return;
        }
        let mut mine = goals::inventory(id).unwrap_or_else(|| data.items.clone());
        let mut moved = 0;
        for slot in theirs.iter_mut() {
            let Some(s) = slot.as_mut() else { continue };
            if goals::armour_part(&s.item).is_some() || matches!(&*s.item, "minecraft:torch" | "minecraft:shield") {
                continue;
            }
            let left = store_some(&mut mine, s, MOSCOW_STACK);
            if left < s.count {
                moved += 1;
                s.count = left;
                if left == 0 {
                    *slot = None;
                }
            }
        }
        if moved == 0 {
            return;
        }
        say(host, id, data, 2, &format!("Took {moved} stacks off it"));
        goals::put_inventory(target, theirs.clone());
        self.store_items(host, target, theirs, false);
        goals::put_inventory(id, mine.clone());
        data.items = mine;
    }

    /// Opens doll `id`'s inventory for `player` as a chest: they put items in and take them out as
    /// in any chest, and what is left in it when it closes is the doll's (`menu_input_closed`).
    /// While it is open the items are the menu's, so a doll that dies meanwhile leaves none to copy.
    /// A wheel turn over a long doll inventory: what the screen holds now is saved to the doll, and
    /// the screen opens again in its place a row (or rows) further on. The old screen's close, if the
    /// server reports it, finds its window already saved and gives nothing back.
    fn scroll_inventory(&self, host: &mut dyn Host, player: u128, id: u32, window: i32, rows: i32) {
        let Some(contents) = host.menu_contents(player) else {
            debug(host, &format!("inventory {id}: scroll with no screen; forgotten"));
            OPEN.lock().ok().map(|mut o| o.remove(&player));
            GRID_OPEN.lock().ok().map(|mut g| g.remove(&player));
            goals::set_chest_open(None);
            return;
        };
        let Some((offset, view)) = GRID_OPEN.lock().ok().and_then(|g| g.get(&player).copied()) else { return };
        let view = view.min(contents.len());
        let mut items = goals::inventory(id).unwrap_or_default();
        for (k, cell) in contents[..view].iter().enumerate() {
            if let Some(slot) = items.get_mut(offset + k) {
                *slot = cell.clone();
            }
        }
        let holder = Holder::Entity(id);
        if contents.len() > view && DOLL.has(host, holder).unwrap_or(false) {
            let mut cells = contents[view..].to_vec();
            let baubles = baubles_from(&mut cells);
            let mut data = DOLL.get(host, holder).unwrap_or_default();
            goals::set_baubles(id, worn_names(&baubles));
            data.baubles = baubles;
            if let Some(d) = doll_of(host, id) {
                remember(id, d, &data);
            }
            let _ = DOLL.set(host, holder, data);
        }
        goals::put_inventory(id, items.clone());
        goals::chest_closed(id, items);
        SAVED_WINDOWS.lock().ok().map(|mut w| w.push((player, window)));
        OPEN.lock().ok().map(|mut o| o.remove(&player));
        GRID_OPEN.lock().ok().map(|mut g| g.remove(&player));
        goals::set_doll_menu(player, window, None);
        if let Ok(mut v) = MENU_VIEW.lock() {
            *v = None;
        }
        let total = doll_of(host, id).map_or(0, |d| d.slots).div_ceil(9);
        if let Ok(mut s) = SCROLL.lock() {
            let top = s.get(&player).copied().unwrap_or(0) as i32 + rows;
            s.insert(player, top.clamp(0, total.saturating_sub(3) as i32) as usize);
        }
        debug(host, &format!("inventory {id}: scrolled {rows} (window {window} saved)"));
        self.open_inventory(host, player, id);
    }

    fn open_inventory(&self, host: &mut dyn Host, player: u128, id: u32) {
        let holder = Holder::Entity(id);
        // A doll screen this player has no menu open for any more: its close was missed. Forgotten,
        // so it doesn't stop this one (its items went back to the player when it closed).
        if OPEN.lock().is_ok_and(|o| o.contains_key(&player)) && host.menu_contents(player).is_none() {
            debug(host, &format!("inventory {id}: a doll screen of this player's was left over; cleared"));
            OPEN.lock().ok().map(|mut o| o.remove(&player));
            GRID_OPEN.lock().ok().map(|mut g| g.remove(&player));
            if let Ok(mut v) = MENU_VIEW.lock() {
                *v = None;
            }
        }
        // A doll held open while no player has a doll screen: the same.
        if goals::chest_open().is_some() && OPEN.lock().is_ok_and(|o| o.is_empty()) {
            debug(host, &format!("inventory {id}: doll {:?} was held open with no screen; released", goals::chest_open()));
            goals::set_chest_open(None);
        }
        let already = OPEN.lock().is_ok_and(|open| open.contains_key(&player));
        let has_data = DOLL.has(host, holder).unwrap_or(false);
        if already || goals::chest_open().is_some() || !has_data {
            debug(host, &format!(
                "inventory {id}: not opened (screen already open {already}, doll held open {:?}, doll data {has_data})",
                goals::chest_open()
            ));
            return;
        }
        let mut data = DOLL.get(host, holder).unwrap_or_default();
        let items = goals::inventory(id).unwrap_or_else(|| data.items.clone());
        let size = items.len();
        let name = doll_of(host, id).map_or("Doll", |d| d.name);
        let switch = has_rock_switch(name) && size <= 45;
        // An inventory too long for the screen with the bauble grid under it (Russia's, Moscow's)
        // shows three rows at a time, scrolled with the mouse wheel.
        let rows_total = size.div_ceil(9);
        let scrolls = rows_total + 3 + usize::from(switch) > 6;
        let view_rows = if scrolls { 3 } else { rows_total };
        let top = if scrolls {
            SCROLL.lock().ok().and_then(|s| s.get(&player).copied()).unwrap_or(0).min(rows_total - view_rows)
        } else {
            0
        };
        let (offset, view) = (top * 9, view_rows * 9);
        let mut shown: Vec<Option<ItemStack>> = items[offset..(offset + view).min(size)].to_vec();
        shown.resize(view, None);
        shown.extend(bauble_block(&data.baubles));
        if switch {
            shown.extend(switch_row(data.keep_rock));
        }
        let rows = (shown.len() / 9).clamp(1, 6);
        let title = if scrolls {
            format!("{name} · rows {}-{} of {rows_total} · scroll", top + 1, top + view_rows)
        } else {
            name.to_owned()
        };
        let input = view + goals::BAUBLE_CELLS;
        let window = host.open_menu_with_input(player, rows as _, Text::literal(&title), shown, 0..input);
        debug(host, &format!("inventory {id}: open_menu_with_input({rows} rows) -> {window:?}"));
        let Some(window) = window else { return };
        if let Ok(mut open) = OPEN.lock() {
            open.insert(player, (id, window));
        }
        goals::set_doll_menu(
            player,
            window,
            Some(goals::MenuLayout { slots: view, offset, total: size, baubles_at: Some(view), moscow: name == goals::MOSCOW }),
        );
        data.baubles = vec![None; goals::BAUBLE_SLOTS];
        goals::set_baubles(id, Vec::new());
        GRID_OPEN.lock().ok().map(|mut g| g.insert(player, (offset, view)));
        OUTLINES_LOGGED.store(false, std::sync::atomic::Ordering::Relaxed);
        if size > goals::ARMOUR_SLOTS + 1 {
            host.action_bar(player, Text::literal("Last 4 slots: armour (helmet last) · the one before: shield · the 3x3 below: baubles"));
        }
        if let Ok(mut v) = MENU_VIEW.lock() {
            *v = Some(MenuView { title: title.clone(), total: size, offset, view, empty: vec![false; view] });
        }
        goals::set_chest_open(Some(id));
        // What the screen shows is the menu's while it is open; the rest stays the doll's.
        data.items = items;
        for slot in data.items.iter_mut().skip(offset).take(view) {
            *slot = None;
        }
        goals::put_inventory(id, data.items.clone());
        if let Some(doll) = doll_of(host, id) {
            remember(id, doll, &data);
        }
        let _ = DOLL.set(host, holder, data);
    }

    /// A doll's 4x4 bauble grid on a screen of its own (Russia's and Moscow's have no room under
    /// their inventories, and any doll's opens this way when right-clicked with a bauble).
    fn open_baubles(&self, host: &mut dyn Host, player: u128, id: u32) {
        let holder = Holder::Entity(id);
        if OPEN.lock().is_ok_and(|o| o.contains_key(&player))
            || BAUBLE_OPEN.lock().is_ok_and(|o| o.contains_key(&player) || o.values().any(|(d, _)| *d == id))
            || !DOLL.has(host, holder).unwrap_or(false)
        {
            return;
        }
        let mut data = DOLL.get(host, holder).unwrap_or_default();
        let name = doll_of(host, id).map_or("Doll", |d| d.name);
        let title = Text::literal(format!("{name}: baubles"));
        let Some(window) = host.open_menu_with_input(player, 3, title, bauble_block(&data.baubles), 0..goals::BAUBLE_CELLS) else { return };
        BAUBLE_OPEN.lock().ok().map(|mut o| o.insert(player, (id, window)));
        goals::set_doll_menu(player, window, Some(goals::MenuLayout { slots: 0, offset: 0, total: 0, baubles_at: Some(0), moscow: false }));
        data.baubles = vec![None; goals::BAUBLE_SLOTS];
        goals::set_baubles(id, Vec::new());
        if let Some(doll) = doll_of(host, id) {
            remember(id, doll, &data);
        }
        let _ = DOLL.set(host, holder, data);
    }

    /// What a doll's inventory holds now, from the chest screen or its own goals: saved with it.
    fn store_items(&self, host: &mut dyn Host, id: u32, items: Vec<Option<ItemStack>>, from_chest: bool) {
        let holder = Holder::Entity(id);
        if !DOLL.has(host, holder).unwrap_or(false) {
            debug(host, &format!("inventory {id}: changed, but the doll is gone"));
            return;
        }
        let mut data = DOLL.get(host, holder).unwrap_or_default();
        data.items = items;
        if doll_of(host, id).is_some_and(|d| d.name == goals::MOSCOW) {
            compact(&mut data.items, MOSCOW_STACK);
            goals::put_inventory(id, data.items.clone());
        }
        if from_chest {
            debug(host, &format!("inventory {id}: chest closed, {} stacks", data.items.iter().flatten().count()));
            goals::put_inventory(id, data.items.clone());
        }
        if let Some(doll) = doll_of(host, id) {
            remember(id, doll, &data);
        }
        let _ = DOLL.set(host, holder, data);
    }
}

/// `canCollectRange`: Russia takes an item within two blocks.
const COLLECT_RANGE_SQR: f64 = 4.0;

/// How many of its own ticks pass between Russia's looks for dropped items.
static COLLECT_TICKS: LazyLock<Mutex<HashMap<u32, u32>>> = LazyLock::new(Default::default);

/// `EntityDollAICollectItem`, the part a mob's goals cannot do: finds the nearest dropped item Russia
/// has room for, for its goal to walk to, and takes whatever is within reach into its inventory.
fn collect(host: &mut dyn Host, id: u32, moscow: bool) {
    let due = COLLECT_TICKS.lock().is_ok_and(|mut ticks| {
        let n = ticks.entry(id).or_default();
        *n = (*n + 1) % 5;
        *n == 0
    });
    if !due {
        return;
    }
    let Some(entity) = host.entity(id) else { return };
    let (Some(at), Some(level)) = (host.entity_position(entity), host.entity_level(entity)) else {
        return;
    };
    let dimension = level.dimension();
    let found = host.item_entities(
        dimension,
        [at[0] - 16.0, at[1] - 8.0, at[2] - 16.0],
        [at[0] + 16.0, at[1] + 8.0, at[2] + 16.0],
    );
    let holder = Holder::Entity(id);
    let mut data = DOLL.get(host, holder).unwrap_or_default();
    // While its owner has it open, what Russia carries is the chest screen's.
    let can_take = goals::chest_open() != Some(id);
    let mut taken = Vec::new();
    let mut nearest: Option<(f64, [f64; 3])> = None;
    for item in found {
        if item.stack.count == 0 || goals::is_doll_item(&item.stack.item) {
            continue;
        }
        let max = if moscow { MOSCOW_STACK } else { host.max_stack_size(&item.stack.item).unwrap_or(64) };
        if room_for(&data.items, &item.stack, max) == 0 {
            continue;
        }
        let d = item.position;
        let d2 = (d[0] - at[0]).powi(2) + (d[1] - at[1]).powi(2) + (d[2] - at[2]).powi(2);
        let reach = if goals::wears(id, "alice_dolls:item_magnet_bauble") { 36.0 } else { COLLECT_RANGE_SQR };
        if can_take && d2 <= reach && item.pickup_delay == 0 {
            let mut items = data.items.clone();
            if moscow {
                compact(&mut items, max);
            }
            let left = store_some(&mut items, &item.stack, max);
            let mut rest = item.stack.clone();
            rest.count = left;
            // Only once the item on the ground is changed does Russia have it: nothing is copied.
            if host.set_item_entity(dimension, item.id, rest) {
                taken.push((item.stack.item.to_string(), item.stack.count - left));
                data.items = items;
            }
            continue;
        }
        if nearest.is_none_or(|(n, _)| d2 < n) {
            nearest = Some((d2, d));
        }
    }
    goals::set_item_target(id, nearest.map(|(_, d)| d));
    if taken.is_empty() {
        return;
    }
    host.play_local_sound("minecraft:entity.item.pickup", at, 0.2, 2.0, 0);
    for (item, count) in &taken {
        let short = item.strip_prefix("minecraft:").unwrap_or(item);
        say(host, id, &data, 3, &format!("{short} x{count}"));
    }
    goals::put_inventory(id, data.items.clone());
    if let Some(doll) = doll_of(host, id) {
        remember(id, doll, &data);
    }
    let _ = DOLL.set(host, holder, data);
}

impl Hooks for Mod {
    /// Places a doll from its item.
    fn use_item_on(&self, host: &mut dyn Host, event: &mut UseItemOnEvent<'_>) -> Flow {
        let item = event.stack.item.to_string();
        if item.starts_with("alice_dolls:") {
            debug(host, &format!("use_item_on {item} at {:?} face {:?}", event.pos, event.face));
        }
        let Some(doll) = doll_for_item(&item) else {
            return Flow::Continue;
        };
        let Some(player) = host.acting_player() else {
            debug(host, &format!("  {item}: no acting player"));
            return Flow::Continue;
        };
        let at = [
            f64::from(event.pos[0] + event.face[0]) + 0.5,
            f64::from(event.pos[1] + event.face[1]),
            f64::from(event.pos[2] + event.face[2]) + 0.5,
        ];
        // What it carried when it was picked up comes back with it. The item goes first, so a doll
        // and its inventory are never in two places.
        let survival = player.game_mode != 1;
        // In survival, the inventory is the one in the stack really taken from the player.
        let taken = if survival {
            match take_placed(host, &player, event.stack) {
                Some(taken) => taken,
                None => {
                    debug(host, &format!("  {item}: could not take it from the player's hand"));
                    return Flow::Cancel;
                }
            }
        } else {
            event.stack.clone()
        };
        let items = items_in(&taken, doll.slots);
        let summoned = host.summon(&player.dimension, KIND, at, doll_nbt(doll));
        debug(host, &format!("  {item}: summon in {} at {at:?} -> {summoned:?}", player.dimension));
        let Some(id) = summoned else {
            if survival {
                // Back where it came from, which it has just left room in.
                let mut back = taken.clone();
                back.count = 1;
                give_to_owner(host, player.uuid, vec![back]);
            }
            let text = format!("{} : there is no room here", doll.name);
            host.action_bar(player.uuid, Text::literal(text));
            return Flow::Cancel;
        };
        let anchor = nearest_anchor(&player.dimension, player.uuid, at);
        let (favour, form_level) = favour_in(&taken);
        let data = DollData {
            owner: player.uuid,
            items,
            baubles: baubles_in(&taken),
            anchor,
            favour,
            form_level,
            ..DollData::default()
        };
        goals::set_level(id, favour_level(favour).0);
        remember(id, doll, &data);
        tell_state(id, &data);
        goals::put_inventory(id, data.items.clone());
        let _ = DOLL.set(host, Holder::Entity(id), data);
        let text = format!("{} : Activation successful!", doll.name);
        host.action_bar(player.uuid, Text::literal(text));
        Flow::Cancel
    }

    /// Marisa's stars and Reimu's amulets, as danmaku facing the camera: a star a yellow glow with a
    /// white core, an amulet a red talisman.
    fn world_submit(&self, host: &dyn Host, event: &mut WorldSubmitEvent<'_>) {
        let shots = goals::flying_shots();
        let beams = goals::doll_beams();
        let marked = HIGHLIGHT
            .lock()
            .ok()
            .and_then(|h| *h)
            .filter(|(_, at)| at.elapsed() < HIGHLIGHT_FOR)
            .and_then(|(id, at)| Some((host.entity_position(host.entity(id)?)?, at.elapsed().as_secs_f64())));
        if shots.is_empty() && beams.is_empty() && marked.is_none() {
            return;
        }
        let Some(camera) = host.camera() else { return };
        let (yaw, pitch) = (f64::from(camera.yaw).to_radians(), f64::from(camera.pitch).to_radians());
        let forward = [-yaw.sin() * pitch.cos(), -pitch.sin(), yaw.cos() * pitch.cos()];
        let right = [-yaw.cos(), 0.0, -yaw.sin()];
        let up = [
            right[1] * forward[2] - right[2] * forward[1],
            right[2] * forward[0] - right[0] * forward[2],
            right[0] * forward[1] - right[1] * forward[0],
        ];
        let draw = &mut *event.draw;
        let partial = f64::from(draw.partial);
        // The doll a Moscow doll is now following: a green light bobbing over it.
        if let Some((at, t)) = marked {
            let centre = [at[0], at[1] + 1.3 + (t * 4.0).sin() * 0.1, at[2]];
            for (half, rgba) in [(0.45, [80u8, 255, 120, 200]), (0.2, [255, 255, 255, 255])] {
                draw.push();
                if draw.translate_world(centre) {
                    let corner = |a: f64, b: f64| [0, 1, 2].map(|i| (right[i] * a + up[i] * b) as f32 * half as f32);
                    let corners = [corner(-1.0, 1.0), corner(-1.0, -1.0), corner(1.0, -1.0), corner(1.0, 1.0)];
                    host.world_quad(draw, "thkaguyamod:entity/shot/light", corners, [0.0, 0.0, 8.0, 16.0], rgba);
                }
                draw.pop();
            }
        }
        // Marisa's beam: a ribbon turned to the camera, a yellow glow round a white core, cut
        // across the middle of the light shot so it glows across its width and runs its length.
        for (from, to) in beams {
            let v = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
            let side = [v[1] * forward[2] - v[2] * forward[1], v[2] * forward[0] - v[0] * forward[2], v[0] * forward[1] - v[1] * forward[0]];
            let n = (side[0] * side[0] + side[1] * side[1] + side[2] * side[2]).sqrt();
            if n < 1e-6 {
                continue;
            }
            for (half, rgba) in [(0.4, [255u8, 220, 70, 170]), (0.16, [255, 255, 255, 240])] {
                let s = side.map(|c| c / n * half);
                let a = [-s[0], -s[1], -s[2]];
                let b = [v[0] - s[0], v[1] - s[1], v[2] - s[2]];
                let c = [v[0] + s[0], v[1] + s[1], v[2] + s[2]];
                let d = s;
                let f = |p: [f64; 3]| p.map(|x| x as f32);
                draw.push();
                if draw.translate_world(from) {
                    host.world_quad(draw, "thkaguyamod:entity/shot/light", [f(a), f(b), f(c), f(d)], [0.0, 7.5, 8.0, 8.5], rgba);
                    host.world_quad(draw, "thkaguyamod:entity/shot/light", [f(d), f(c), f(b), f(a)], [8.0, 7.5, 0.0, 8.5], rgba);
                }
                draw.pop();
            }
        }
        for (kind, at, was) in shots {
            let centre = [0, 1, 2].map(|i| was[i] + (at[i] - was[i]) * partial);
            // Touhou Items Mod's own danmaku: Marisa's a yellow star-light, Reimu's a red talisman.
            let quads: &[(&str, [f32; 4], f64, [u8; 4])] = if kind == 0 {
                &[
                    ("thkaguyamod:entity/shot/light", [0.0, 0.0, 8.0, 16.0], 0.45, [255, 220, 60, 210]),
                    ("thkaguyamod:entity/shot/light", [0.0, 0.0, 8.0, 16.0], 0.22, [255, 255, 255, 255]),
                    ("thkaguyamod:entity/shot/small", [6.0, 0.0, 8.0, 16.0], 0.18, [255, 255, 255, 255]),
                ]
            } else {
                &[("thkaguyamod:entity/shot/talisman", [0.0, 0.0, 2.0, 16.0], 0.3, [255, 255, 255, 255])]
            };
            for &(sprite, uv, half, rgba) in quads {
                draw.push();
                if draw.translate_world(centre) {
                    let corner = |a: f64, b: f64| [0, 1, 2].map(|i| (right[i] * a + up[i] * b) as f32 * half as f32);
                    let corners = [corner(-1.0, 1.0), corner(-1.0, -1.0), corner(1.0, -1.0), corner(1.0, 1.0)];
                    host.world_quad(draw, sprite, corners, uv, rgba);
                }
                draw.pop();
            }
        }
    }

    /// `EntityAliceDoll.interact`: what a right-click does depends on who, and with what.
    fn interact_entity(&self, host: &mut dyn Host, event: &mut InteractEntityEvent<'_>) -> Flow {
        if event.kind != KIND {
            return Flow::Continue;
        }
        let held = event.stack.map(|stack| stack.item.to_string());
        let stack = event.stack.cloned();
        self.interact_doll(host, event.entity as u32, held.as_deref(), stack.as_ref())
    }

    /// A right-click with an item aimed at a Goliath's body, which the small box it stands on misses.
    fn use_item(&self, host: &mut dyn Host, event: &mut UseItemEvent<'_>) -> Flow {
        let item = event.stack.item.to_string();
        if item == DOLL_BELL {
            return self.ring_bell(host);
        }
        let armour = goals::armour_part(&item).is_some();
        if !armour && !["minecraft:chest", DOLL_CORE, "minecraft:book", "minecraft:lead"].contains(&item.as_str()) {
            return Flow::Continue;
        }
        let Some(player) = host.acting_player() else { return Flow::Continue };
        let stack = event.stack.clone();
        match goliath_in_sight(host, &player) {
            Some(id) => self.interact_doll(host, id, Some(&item), Some(&stack)),
            None => Flow::Continue,
        }
    }

    /// A doll's chest menu closed: what is left in it is the doll's, saved on the next tick. If the
    /// doll is gone by then, the items stay in the menu and go back to the player.
    fn menu_input_closed(&self, host: &mut dyn Host, event: &mut MenuInputClosedEvent<'_>) {
        // A screen of a doll's baubles alone.
        if let Some((id, window)) = BAUBLE_OPEN.lock().ok().and_then(|o| o.get(&event.uuid).copied())
            && window == event.window
        {
            BAUBLE_OPEN.lock().ok().map(|mut o| o.remove(&event.uuid));
            goals::set_doll_menu(event.uuid, window, None);
            if known(id).is_none() {
                // The doll went: the baubles go back to the player, not the panes.
                for (i, cell) in event.slots.iter_mut().enumerate() {
                    if goals::bauble_cell(i, 0) == Some(None) {
                        *cell = None;
                    }
                }
                return;
            }
            let baubles = baubles_from(event.slots);
            BAUBLES_CLOSED.lock().ok().map(|mut c| c.push((id, baubles)));
            return;
        }
        let saved = SAVED_WINDOWS.lock().is_ok_and(|mut w| {
            let n = w.len();
            w.retain(|&(p, win)| !(p == event.uuid && win == event.window));
            w.len() != n
        });
        if saved {
            // Scrolled away from: everything in it is in the doll already.
            event.slots.iter_mut().for_each(|c| *c = None);
            return;
        }
        let Some((id, window)) = OPEN.lock().ok().and_then(|open| open.get(&event.uuid).copied()) else {
            return;
        };
        if window != event.window {
            return;
        }
        if let Ok(mut open) = OPEN.lock() {
            open.remove(&event.uuid);
        }
        goals::set_doll_menu(event.uuid, window, None);
        if let Ok(mut v) = MENU_VIEW.lock() {
            *v = None;
        }
        let size = doll_of(host, id).map_or(event.slots.len(), |d| d.slots);
        let (offset, view) = GRID_OPEN.lock().ok().and_then(|mut g| g.remove(&event.uuid)).unwrap_or_else(|| {
            // Its layout missing: read as the whole inventory, no grid (the doll is let go either way).
            (0, size.min(event.slots.len()))
        });
        let view = view.min(event.slots.len());
        if known(id).is_none() {
            goals::set_chest_open(None);
            debug(host, &format!("inventory {id}: closed after the doll went; the items go back"));
            for (i, cell) in event.slots.iter_mut().enumerate().skip(view) {
                if goals::bauble_cell(i, view) == Some(None) {
                    *cell = None;
                }
            }
            return;
        }
        if event.slots.len() > view {
            let baubles = baubles_from(&mut event.slots[view..]);
            BAUBLES_CLOSED.lock().ok().map(|mut c| c.push((id, baubles)));
        }
        // The rows it showed, back among the doll's others.
        let mut items = goals::inventory(id).unwrap_or_default();
        for (k, cell) in event.slots[..view].iter_mut().enumerate() {
            if let Some(slot) = items.get_mut(offset + k) {
                *slot = cell.take();
            }
        }
        goals::chest_closed(id, items);
    }

    /// The outlines of a doll's armour and shield slots, where they are empty, as vanilla draws them
    /// in the player's own inventory and Touhou Little Maid in its maid's.
    fn screen_render_post(&self, host: &dyn Host, event: &mut ScreenRenderPostEvent<'_>) {
        let Some(MenuView { title: name, total: size, offset, view, empty }) = MENU_VIEW.lock().ok().and_then(|v| v.clone()) else {
            return;
        };
        if event.screen.title.is_none_or(|t| t.plain() != name) || size <= goals::ARMOUR_SLOTS {
            return;
        }
        let [x, y, _, _] = event.screen.rect;
        let first = !OUTLINES_LOGGED.swap(true, std::sync::atomic::Ordering::Relaxed);
        let slots = [
            (goals::armour_slot(size, 0), "helmet"),
            (goals::armour_slot(size, 1), "chestplate"),
            (goals::armour_slot(size, 2), "leggings"),
            (goals::armour_slot(size, 3), "boots"),
            (goals::shield_slot(size), "shield"),
        ];
        for (d, sprite) in slots {
            // The doll's slot `d`, where this screen shows it, if it does.
            let Some(i) = d.checked_sub(offset).filter(|&i| i < view) else { continue };
            if empty.get(i) != Some(&true) {
                continue;
            }
            let (sx, sy) = (x + 8.0 + 18.0 * (i % 9) as f32, y + 18.0 + 18.0 * (i / 9) as f32);
            let drawn = host.sprite(event.batch, &format!("gui/sprites/container/slot/{sprite}"), [0.0, 0.0, 16.0, 16.0], [sx, sy, 16.0, 16.0]);
            if first {
                debug(host, &format!("outline {sprite} at slot {i} ({sx},{sy}) of {name}: drawn {drawn}"));
            }
        }
    }

    /// The mouse wheel over a doll screen that scrolls: a row up or down.
    fn screen_mouse_scrolled_pre(&self, _: &dyn Host, event: &mut ScreenMouseScrolledPreEvent<'_>) -> Flow {
        let Some(view) = MENU_VIEW.lock().ok().and_then(|v| v.clone()) else { return Flow::Continue };
        let scrolls = view.view < view.total;
        if !scrolls || event.screen.title.is_none_or(|t| t.plain() != view.title) || event.scroll[1] == 0.0 {
            return Flow::Continue;
        }
        let rows = if event.scroll[1] > 0.0 { -1 } else { 1 };
        if let Ok(mut s) = SCROLLED.lock() {
            s.push((0, rows));
        }
        Flow::Cancel
    }

    /// Aya's photos worth keeping: a screenshot each, as the original saved.
    fn client_tick_post(&self, host: &mut dyn Host, _: &mut ClientTickPostEvent) {
        // Her photo taken: the HUD as it was.
        let pov_over = AYA_POV.lock().is_ok_and(|p| p.is_none());
        if pov_over && let Some(mode) = AYA_HUD.lock().ok().and_then(|mut h| h.take()) {
            host.set_hud_mode(mode);
        }
        for photo in goals::take_photos() {
            // The next frames are drawn from Aya's eyes with no HUD or hand, and the screenshot
            // is of the first.
            if let Ok(mut pov) = AYA_POV.lock() {
                *pov = Some((photo, AYA_POV_FRAMES));
            }
            if let Ok(mut saved) = AYA_HUD.lock()
                && saved.is_none()
            {
                *saved = Some(host.hud_mode());
            }
            host.set_hud_mode(HudMode::AllHidden);
            match host.take_screenshot() {
                Ok(path) => debug(host, &format!("aya photo: {path}")),
                Err(e) => debug(host, &format!("aya photo: {e}")),
            }
        }
    }

    /// A doll's name tag carries its favour level and how far it is to the next.
    fn render_name_tag(&self, host: &dyn Host, event: &mut RenderNameTagEvent<'_>) {
        if event.kind != KIND {
            return;
        }
        let Some((level, into, need)) = FAVOUR_SHOWN.lock().ok().and_then(|s| s.get(&event.entity_id).copied()) else { return };
        let Some(doll) = doll_of(host, event.entity_id) else { return };
        *event.text = Some(favour_tag(doll.name, level, into, need));
    }

    /// While Aya's photo is being taken, the camera is hers.
    fn camera_pose(&self, _host: &dyn Host, event: &mut CameraPoseEvent<'_>) {
        let Ok(mut pov) = AYA_POV.lock() else { return };
        let Some((photo, frames)) = pov.as_mut() else { return };
        *event.position = photo.eye;
        event.yaw = photo.yaw;
        event.pitch = photo.pitch;
        *frames -= 1;
        if *frames <= 0 {
            *pov = None;
        }
    }

    /// The keep-or-void switch in a Diamond or Netherite Horai's inventory screen.
    fn menu_clicked(&self, host: &mut dyn Host, event: &mut MenuClickedEvent) {
        if let Some((doll, window, players)) = PICKING.lock().ok().and_then(|p| p.get(&event.uuid).cloned())
            && window == event.window
        {
            let Some((target, name)) = usize::try_from(event.slot).ok().and_then(|i| players.get(i).cloned()) else { return };
            PICKING.lock().ok().map(|mut p| p.remove(&event.uuid));
            host.close_menu(event.uuid);
            if let Ok(mut m) = MISSIONS.lock() {
                m.insert(doll, Mission { owner: event.uuid, target, ticks: 0, phase: Phase::Stalk });
            }
            goals::set_mission(doll, true);
            let data = DOLL.get(host, Holder::Entity(doll)).unwrap_or_default();
            say(host, doll, &data, 1, &format!("Going to see {name}..."));
            return;
        }
        let Some((id, window)) = OPEN.lock().ok().and_then(|open| open.get(&event.uuid).copied()) else { return };
        if window != event.window {
            return;
        }
        let Some(doll) = doll_of(host, id) else { return };
        if !has_rock_switch(doll.name) || event.slot != (doll.slots + SWITCH_AT) as i32 {
            return;
        }
        let holder = Holder::Entity(id);
        let mut data = DOLL.get(host, holder).unwrap_or_default();
        data.keep_rock = !data.keep_rock;
        goals::set_keep_rock(id, data.keep_rock);
        let mut shown = vec![None; doll.slots];
        shown.extend(switch_row(data.keep_rock));
        host.set_menu_items(event.uuid, shown);
        if let Some(at) = host.entity(id).and_then(|e| host.entity_position(e)) {
            host.play_local_sound("minecraft:ui.button.click", at, 0.6, 1.0, 0);
        }
        let _ = DOLL.set(host, holder, data);
    }

    /// Sneak and right-click a doll anchor: its radius grows by ten blocks, up to a hundred, then
    /// starts again at thirty.
    fn right_click_block(&self, host: &mut dyn Host, event: &mut RightClickBlockEvent) -> Flow {
        let Some(player) = host.acting_player() else { return Flow::Continue };
        if !player.sneaking {
            return Flow::Continue;
        }
        let Some(level) = host.level(&player.dimension) else { return Flow::Continue };
        if !host.level_block(level, event.pos).is_some_and(|b| b.starts_with(ANCHOR)) {
            return Flow::Continue;
        }
        if !fresh_click(player.uuid) {
            return Flow::Cancel;
        }
        let owner = ANCHORS.lock().ok().and_then(|a| a.iter().find(|x| x.1 == event.pos).map(|x| x.2));
        if owner.is_some_and(|o| o != player.uuid) {
            host.action_bar(player.uuid, Text::literal("This doll anchor isn't yours"));
            return Flow::Cancel;
        }
        // One placed before anchors were listed is taken on now, by whoever adjusts it.
        if owner.is_none()
            && let Ok(mut a) = ANCHORS.lock()
        {
            a.push((player.dimension.clone(), event.pos, player.uuid));
        }
        let now = radius_of(event.pos);
        let next = if now >= ANCHOR_RADIUS_MAX { ANCHOR_RADIUS } else { now + 10 };
        RADII.lock().ok().map(|mut r| r.insert(event.pos, next));
        ANCHORS_CHANGED.store(true, std::sync::atomic::Ordering::Relaxed);
        host.action_bar(player.uuid, Text::literal(format!("Doll anchor radius: {next} blocks")));
        let at = event.pos.map(|c| f64::from(c) + 0.5);
        host.play_local_sound("minecraft:block.note_block.chime", at, 0.6, 0.5 + next as f32 / 100.0, 0);
        Flow::Cancel
    }

    /// Every block a player places: no doll will dig it.
    fn place_block(&self, host: &mut dyn Host, event: &mut PlaceBlockEvent<'_>) {
        goals::mark_placed(event.pos);
        if &*event.stack.item == ANCHOR
            && let Some(player) = host.acting_player()
            && let Ok(mut a) = ANCHORS.lock()
        {
            a.retain(|x| x.1 != event.pos);
            a.push((player.dimension.clone(), event.pos, player.uuid));
            RADII.lock().ok().map(|mut r| r.remove(&event.pos));
            ANCHORS_CHANGED.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    fn menu_closed(&self, _: &mut dyn Host, event: &mut MenuClosedEvent) {
        if let Ok(mut p) = PICKING.lock()
            && p.get(&event.uuid).is_some_and(|(_, w, _)| *w == event.window)
        {
            p.remove(&event.uuid);
        }
        if let Ok(mut o) = BAUBLE_OPEN.lock()
            && o.get(&event.uuid).is_some_and(|(_, w)| *w == event.window)
        {
            o.remove(&event.uuid);
        }
        if let Ok(mut open) = OPEN.lock()
            && open.get(&event.uuid).is_some_and(|(_, window)| *window == event.window)
        {
            open.remove(&event.uuid);
        }
        goals::set_doll_menu(event.uuid, event.window, None);
    }

    /// `attackEntityFrom`: a blow with a Doll Core in hand picks the doll straight up, with
    /// everything it carried.
    fn attack_entity(&self, host: &mut dyn Host, event: &mut AttackEntityEvent<'_>) -> Flow {
        let id = event.entity as u32;
        // Ooedo, punched with an empty hand: it doesn't take the blow, it goes off.
        if event.kind == KIND
            && host.held().is_none()
            && known(id).is_some_and(|k| k.doll.name == goals::OOEDO)
        {
            goals::detonate(id);
            return Flow::Cancel;
        }
        // Aya, punched with an empty hand: no blow, a photo of whoever threw it.
        if event.kind == KIND && host.held().is_none() && known(id).is_some_and(|k| k.doll.name == goals::AYA) {
            goals::snap(id);
            return Flow::Cancel;
        }
        // Hit by its own owner: the food it eats to heal that earns it no love.
        if event.kind == KIND
            && let Some(player) = host.acting_player()
            && known(id).is_some_and(|k| k.owner == player.uuid)
            && host.held().is_none_or(|(item, _, _)| item != DOLL_CORE)
        {
            goals::owner_hurt(id);
        }
        if event.kind != KIND
            || !DOLL.has(host, Holder::Entity(id)).unwrap_or(false)
            || host.held().is_none_or(|(item, _, _)| item != DOLL_CORE)
        {
            return Flow::Continue;
        }
        let (Some(player), Some(doll)) = (host.acting_player(), doll_of(host, id)) else {
            return Flow::Continue;
        };
        let data = DOLL.get(host, Holder::Entity(id)).unwrap_or_default();
        forget(id);
        host.remove_mob(&player.dimension, id);
        if data.xp > 0 {
            host.give_experience(player.uuid, data.xp);
        }
        let items = goals::inventory(id).unwrap_or(data.items);
        let k = Known {
            doll,
            owner: player.uuid,
            items,
            baubles: data.baubles.clone(),
            favour: data.favour,
            form_level: data.form_level,
        };
        give_to_owner(host, player.uuid, belongings(&k));
        Flow::Cancel
    }

    fn entity_tick_listened(&self, _: &mut dyn Host, event: &mut EntityTickListenedEvent) {
        event.listened = true;
    }

    /// Tells the goals each doll's state, and passes on what the dolls said.
    fn entity_tick_post(&self, host: &mut dyn Host, event: &mut EntityTickPostEvent<'_>) {
        // The blocks players placed, kept in the config folder between games, saved every five seconds.
        use std::sync::atomic::Ordering::Relaxed;
        if !ANCHORS_LOADED.swap(true, Relaxed)
            && let Ok(dir) = host.config_dir()
            && let Ok(text) = std::fs::read_to_string(dir.join(ANCHORS_FILE))
            && let Ok(mut a) = ANCHORS.lock()
        {
            for line in text.lines() {
                let w: Vec<&str> = line.split_whitespace().collect();
                let (base, radius) = match w[..] {
                    [d, x, y, z, o, r] => ([d, x, y, z, o], r.parse::<u32>().ok()),
                    [d, x, y, z, o] => ([d, x, y, z, o], None),
                    _ => continue,
                };
                let [dim, x, y, z, owner] = base;
                if let (Ok(x), Ok(y), Ok(z), Ok(owner)) = (x.parse(), y.parse(), z.parse(), u128::from_str_radix(owner, 16)) {
                    a.push((dim.to_owned(), [x, y, z], owner));
                    if let Some(r) = radius {
                        RADII.lock().ok().map(|mut m| m.insert([x, y, z], r));
                    }
                }
            }
        }
        if ANCHORS_CHANGED.swap(false, Relaxed)
            && let Ok(dir) = host.config_dir()
            && let Ok(a) = ANCHORS.lock()
        {
            let text: String = a.iter().map(|(d, p, o)| format!("{d} {} {} {} {o:x} {}\n", p[0], p[1], p[2], radius_of(*p))).collect();
            let _ = std::fs::write(dir.join(ANCHORS_FILE), text);
        }
        if !PLACED_LOADED.swap(true, Relaxed)
            && let Ok(dir) = host.config_dir()
            && let Ok(text) = std::fs::read_to_string(dir.join(PLACED_FILE))
        {
            goals::load_placed(&text);
        }
        if PLACED_TICKS.fetch_add(1, Relaxed) % 100 == 0
            && let Some(text) = goals::placed_changes()
            && let Ok(dir) = host.config_dir()
        {
            let _ = std::fs::write(dir.join(PLACED_FILE), text);
        }
        for (from, level, text) in goals::take_messages() {
            let data = DOLL.get(host, Holder::Entity(from)).unwrap_or_default();
            // On its owner's head, a doll keeps its reports to itself: only what must be said is.
            if data.mode == goals::RIDE && level > 0 {
                continue;
            }
            say(host, from, &data, level, &text);
        }
        // A wheel turn on a long doll inventory: the screen closes (keeping what it showed) and opens
        // again a row further on.
        let turns: Vec<(u128, i32)> = SCROLLED.lock().map(|mut s| std::mem::take(&mut *s)).unwrap_or_default();
        let rows: i32 = turns.iter().map(|(_, r)| r).sum();
        if !turns.is_empty()
            && let Some((player, (doll, window))) = OPEN.lock().ok().and_then(|o| o.iter().next().map(|(p, v)| (*p, *v)))
        {
            self.scroll_inventory(host, player, doll, window, rows);
        }
        // Reopened once the old screen has really closed (and saved what it showed).
        let waiting: Vec<(u128, u32, u8)> = REOPEN.lock().map(|mut r| std::mem::take(&mut *r)).unwrap_or_default();
        for (player, doll, tries) in waiting {
            let still_open = OPEN.lock().is_ok_and(|o| o.contains_key(&player)) || goals::chest_open().is_some();
            if !still_open {
                self.open_inventory(host, player, doll);
            } else if tries < 40
                && let Ok(mut r) = REOPEN.lock()
            {
                r.push((player, doll, tries + 1));
            }
        }
        let pick: Vec<(u128, u32)> = PENDING_GUIGNOL.lock().map(|mut p| std::mem::take(&mut *p)).unwrap_or_default();
        for (owner, doll) in pick {
            self.open_guignol_panel(host, owner, doll);
        }
        if MISSIONS.lock().is_ok_and(|m| m.contains_key(&event.entity)) {
            self.mission_tick(host, event.entity);
        }
        let asked: Vec<(u128, u32)> = PENDING_BAUBLES.lock().map(|mut p| std::mem::take(&mut *p)).unwrap_or_default();
        for (player, doll) in asked {
            self.open_baubles(host, player, doll);
        }
        let closed: Vec<(u32, Vec<Option<ItemStack>>)> = BAUBLES_CLOSED.lock().map(|mut c| std::mem::take(&mut *c)).unwrap_or_default();
        for (doll, baubles) in closed {
            let holder = Holder::Entity(doll);
            if DOLL.has(host, holder).unwrap_or(false) {
                let mut data = DOLL.get(host, holder).unwrap_or_default();
                goals::set_baubles(doll, worn_names(&baubles));
                data.baubles = baubles;
                if let Some(d) = doll_of(host, doll) {
                    remember(doll, d, &data);
                }
                let _ = DOLL.set(host, holder, data);
            }
        }
        // Wakasagi's catches, into her inventory; what doesn't fit, to her owner.
        for (doll, item) in goals::take_catches() {
            let Some(mut items) = goals::inventory(doll) else { continue };
            let stack = ItemStack::new(item, 1);
            let max = host.max_stack_size(item).unwrap_or(64);
            if store_some(&mut items, &stack, max) > 0
                && let Some(k) = known(doll)
            {
                give_to_owner(host, k.owner, vec![stack]);
            }
            goals::put_inventory(doll, items.clone());
            self.store_items(host, doll, items, false);
        }
        let pending: Vec<(u128, u32)> = PENDING.lock().map(|mut p| std::mem::take(&mut *p)).unwrap_or_default();
        for (player, doll) in pending {
            self.open_inventory(host, player, doll);
        }
        // What the chest menu left in a doll, and what the dolls' own jobs changed.
        let owners = goals::guarded_owners();
        if !owners.is_empty() {
            let players = host.server_players();
            for owner in owners {
                goals::set_owner_at(owner, players.iter().find(|p| p.uuid == owner).map(|p| p.position));
            }
        }
        for (from, at, healed, love) in goals::take_eaten() {
            if let Some(level) = host.entity(from).and_then(|e| host.entity_level(e)) {
                host.level_play_sound(level, "minecraft:entity.generic.eat", at, 6, 0.6, 1.3);
            }
            let holder = Holder::Entity(from);
            if love && DOLL.has(host, holder).unwrap_or(false) {
                let mut data = DOLL.get(host, holder).unwrap_or_default();
                self.add_favour(host, from, &mut data, healed.round().max(1.0) as u32);
                let _ = DOLL.set(host, holder, data);
            }
        }
        for (from, items) in goals::take_chest_closed() {
            self.store_items(host, from, items, true);
        }
        for (from, items) in goals::take_inventory_changes() {
            self.store_items(host, from, items, false);
        }
        let done = HIGHLIGHT.lock().ok().and_then(|mut h| match *h {
            Some((id, at)) if at.elapsed() > HIGHLIGHT_FOR => h.take().map(|(id, _)| id).filter(|_| id != 0),
            _ => None,
        });
        if let Some(id) = done {
            set_glowing(host, id, false);
        }
        if event.kind != KIND {
            return;
        }
        let id = event.entity;
        let holder = Holder::Entity(id);
        if !DOLL.has(host, holder).unwrap_or(false) {
            return;
        }
        let mut data = DOLL.get(host, holder).unwrap_or_default();
        if known(id).is_none_or(|k| k.owner != data.owner) {
            let Some(doll) = doll_of(host, id) else {
                if first_time(&TICK_SEEN, id) {
                    let name = host.entity(id).and_then(|e| host.entity_name(e));
                    debug(host, &format!("tick {id}: has doll data but is named {name:?}"));
                }
                return;
            };
            if first_time(&TICK_SEEN, id) {
                debug(host, &format!("tick {id} {:?}: mode {} owner set {}", doll.name, data.mode, data.owner != 0));
            }
            // A doll saved before its inventory grew gets the rest of its slots, and what it wore
            // (its last four, and the shield before them) moves to its new last ones.
            if data.items.len() < doll.slots {
                let old = data.items.len();
                data.items.resize(doll.slots, None);
                if old > goals::ARMOUR_SLOTS + 1 && doll.slots > old {
                    for k in 0..=goals::ARMOUR_SLOTS {
                        data.items.swap(old - 1 - k, doll.slots - 1 - k);
                    }
                }
                let _ = DOLL.set(host, holder, data.clone());
            }
            // A Goliath left riding by an older version gets down.
            if is_goliath(doll.name) && data.mode == goals::RIDE {
                data.mode = goals::STANDBY;
                let _ = DOLL.set(host, holder, data.clone());
            }
            goals::put_inventory(id, data.items.clone());
            remember(id, doll, &data);
            // The first version stood by with NoAI, which would stop the goals now.
            if let Some(entity) = host.entity(id) {
                host.entity_merge_nbt(entity, [("NoAI".into(), Tag::Byte(0))].into());
            }
        }
        if goals::inventory(id).is_none() {
            goals::put_inventory(id, data.items.clone());
        }
        tell_state(id, &data);
        let (level, into, need) = favour_level(data.favour);
        goals::set_level(id, level);
        if let Ok(mut shown) = FAVOUR_SHOWN.lock() {
            shown.insert(id, (level, into, need));
        }
        goals::set_keep_rock(id, data.keep_rock);
        if GRID_OPEN.lock().is_ok_and(|g| g.is_empty()) && !BAUBLE_OPEN.lock().is_ok_and(|o| o.values().any(|(d, _)| *d == id)) {
            goals::set_baubles(id, worn_names(&data.baubles));
        }
        // Aya on her owner's head: Swiftness II while she rides.
        if data.enabled
            && data.mode == goals::RIDE
            && known(id).is_some_and(|k| k.doll.name == goals::AYA)
            && host.entity(id).is_some_and(|_| ANCHOR_TICKS.lock().is_ok_and(|t| t.get(&id).copied().unwrap_or(0) == 0))
        {
            let speed = NativeEffect { id: "minecraft:speed".into(), amplifier: 1, duration: 60, ambient: true, particles: false };
            host.add_effect(data.owner, &speed);
        }
        let viewer = OPEN.lock().ok().and_then(|o| o.iter().find(|(_, (d, _))| *d == id).map(|(p, _)| *p));
        if let Some(player) = viewer
            && let Some(contents) = host.menu_contents(player)
            && let Ok(mut v) = MENU_VIEW.lock()
            && let Some(view) = v.as_mut()
        {
            view.empty = (0..view.view).map(|i| contents.get(i).is_none_or(Option::is_none)).collect();
        }
        if let Some(uuid) = host.entity(id).and_then(|e| host.entity_uuid(e))
            && let Ok(mut u) = UUIDS.lock()
        {
            u.insert(uuid, id);
        }
        let due = ANCHOR_TICKS.lock().is_ok_and(|mut t| {
            let n = t.entry(id).or_default();
            *n = (*n + 1) % 20;
            *n == 0
        });
        if due {
            self.anchor_work(host, id, &mut data);
            if absorb_xp(host, id, &mut data) {
                let _ = DOLL.set(host, holder, data.clone());
            }
        }
        let moscow = known(id).is_some_and(|k| k.doll.name == goals::MOSCOW);
        // Russia follows the doll she's bound to as Moscow does (but takes nothing from it).
        let russia = known(id).is_some_and(|k| k.doll.name == goals::RUSSIA);
        if russia {
            let to = (data.bound != 0).then(|| UUIDS.lock().ok().and_then(|u| u.get(&data.bound).copied())).flatten();
            goals::set_bound(id, to);
        }
        if known(id).is_some_and(|k| k.doll.name == goals::GRAND_GUIGNOL) {
            cook(host, id);
        }
        // Every doll at work picks up what is dropped around it.
        if data.enabled && matches!(data.mode, goals::PATROL | goals::FOLLOW | goals::SECRET | goals::SECRET_ALL) {
            collect(host, id, moscow);
        } else {
            goals::set_item_target(id, None);
        }
        if moscow {
            let to = (data.bound != 0).then(|| UUIDS.lock().ok().and_then(|u| u.get(&data.bound).copied())).flatten();
            goals::set_bound(id, to);
            let due = MOSCOW_TICKS.lock().is_ok_and(|mut t| {
                let n = t.entry(id).or_default();
                *n = (*n + 1) % 20;
                *n == 0
            });
            if due && to.is_some() && data.enabled && data.mode != goals::STANDBY {
                self.take_from_bound(host, id, &mut data);
                if let Some(doll) = doll_of(host, id) {
                    remember(id, doll, &data);
                }
                let _ = DOLL.set(host, holder, data);
            }
        }
    }

    /// A dead doll leaves itself and what it carried, not feathers and raw chicken, and they go
    /// straight to its owner. They drop where it died only when the owner is away or has no room.
    fn loot_drops(&self, host: &mut dyn Host, event: &mut LootDropsEvent<'_>) {
        if let Some((pos, _)) = event.context.block {
            // Broken, it is no longer a placed block, nor an anchor.
            goals::unmark_placed(pos);
            if event.context.table.contains("doll_anchor")
                && let Ok(mut a) = ANCHORS.lock()
            {
                a.retain(|x| x.1 != pos);
                RADII.lock().ok().map(|mut r| r.remove(&pos));
                ANCHORS_CHANGED.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            // A block the player broke: a Diamond or Netherite Horai on their head digs on.
            if event.context.player.is_some() {
                goals::player_dug(pos);
            }
        }
        let Some((id, KIND)) = event.context.entity else {
            return;
        };
        if let Some(k) = forget(id) {
            event.drops.clear();
            let left = give_to_owner(host, k.owner, belongings(&k));
            event.drops.extend(left);
        }
    }

    /// Ooedo walks away from its own blasts, and from everyone else's.
    fn living_incoming_damage(&self, event: &mut LivingIncomingDamageEvent<'_>) -> Flow {
        // No doll suffocates: one on its owner's head in a low tunnel, or digging, is inside a block.
        if event.kind == KIND
            && event.damage_type.ends_with("in_wall")
            && u32::try_from(event.entity).ok().and_then(known).is_some()
        {
            return Flow::Cancel;
        }
        // Ooedo, for ten seconds after it goes off, takes no harm at all.
        if event.kind == KIND
            && u32::try_from(event.entity).is_ok_and(goals::just_blasted)
        {
            return Flow::Cancel;
        }
        if event.kind == KIND
            && u32::try_from(event.entity).ok().and_then(known).is_some_and(|k| k.doll.name == goals::GRAND_GUIGNOL)
        {
            return Flow::Cancel;
        }
        // Baubles: each turns away its kind of harm (Touhou Little Maid's `onInjured`).
        if event.kind == KIND
            && let Ok(id) = u32::try_from(event.entity)
            && known(id).is_some()
        {
            let ty = event.damage_type.strip_prefix("minecraft:").unwrap_or(event.damage_type);
            let projectile = matches!(
                ty,
                "arrow" | "trident" | "mob_projectile" | "fireball" | "unattributed_fireball" | "wither_skull" | "thrown" | "spit" | "wind_charge" | "fireworks"
            );
            let guards = |bauble: &str| match bauble {
                "alice_dolls:fire_protect_bauble" => matches!(ty, "in_fire" | "on_fire" | "lava" | "hot_floor" | "campfire" | "fireball" | "unattributed_fireball"),
                "alice_dolls:explosion_protect_bauble" => matches!(ty, "explosion" | "player_explosion" | "fireworks" | "bad_respawn_point"),
                "alice_dolls:projectile_protect_bauble" => projectile,
                "alice_dolls:magic_protect_bauble" => matches!(ty, "magic" | "indirect_magic" | "wither" | "dragon_breath" | "sonic_boom"),
                "alice_dolls:fall_protect_bauble" => matches!(ty, "fall" | "stalagmite"),
                "alice_dolls:drown_protect_bauble" => ty == "drown",
                // NimbleFabricBauble: a shot it dodges by teleporting.
                "alice_dolls:nimble_fabric" => projectile,
                _ => false,
            };
            let worn = goals::baubles(id);
            if let Some(b) = worn.iter().find(|b| guards(b)) {
                // NimbleFabricBauble: a shot it dodges by teleporting.
                if &**b == "alice_dolls:nimble_fabric" {
                    goals::request_dodge(id);
                }
                return Flow::Cancel;
            }
        }
        // A doll with a shield blocks three blows in four from a mob or a shot.
        if event.kind == KIND
            && (event.causing > 0 || event.direct > 0)
            && let Ok(id) = u32::try_from(event.entity)
            && known(id).is_some()
            && goals::has_shield(id)
        {
            let mut x = SHIELD_ROLL.load(std::sync::atomic::Ordering::Relaxed);
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            SHIELD_ROLL.store(x, std::sync::atomic::Ordering::Relaxed);
            if x % 4 != 0 {
                return Flow::Cancel;
            }
        }
        // No doll takes a fox's bite.
        if event.kind == KIND
            && u32::try_from(event.entity).ok().and_then(known).is_some()
            && u32::try_from(event.causing).is_ok_and(goals::is_fox)
        {
            return Flow::Cancel;
        }
        if event.kind != KIND || !event.damage_type.ends_with("explosion") {
            return Flow::Continue;
        }
        let ooedo = u32::try_from(event.entity).ok().and_then(known).is_some_and(|k| k.doll.name == goals::OOEDO);
        if ooedo { Flow::Cancel } else { Flow::Continue }
    }

    /// The Ultramarine Orb Elixir and the Totem of Undying in a doll's bauble slot: it doesn't die,
    /// once, and the bauble is used up (Touhou Little Maid's `ExtraLifeBauble`, `UndyingTotemBauble`).
    fn living_death(&self, host: &mut dyn Host, event: &mut LivingDeathEvent<'_>) -> Flow {
        let id = event.entity;
        // A mob with a head in the game, killed by a doll: it leaves its head, which the doll picks up.
        if let Some(head) = mob_head(event.kind)
            && event.attacker != 0
            && known(event.attacker).is_some()
            && let Some(e) = host.entity(id)
            && let (Some(at), Some(level)) = (host.entity_position(e), host.entity_level(e))
        {
            let dimension = level.dimension().to_owned();
            let item: Compound = [("id".into(), Tag::String(head.into())), ("count".into(), Tag::Int(1))].into();
            let nbt: Compound = [("Item".into(), Tag::Compound(item)), ("PickupDelay".into(), Tag::Short(10))].into();
            host.summon(&dimension, "minecraft:item", [at[0], at[1] + 0.5, at[2]], nbt);
        }
        if event.kind == KIND
            && known(id).is_some_and(|k| k.doll.name == goals::GRAND_GUIGNOL)
            && let Some(dimension) = host.entity(id).and_then(|e| host.entity_level(e)).map(|l| l.dimension().to_owned())
        {
            host.set_mob_health(&dimension, id, HEALTH);
            return Flow::Cancel;
        }
        if event.kind != KIND || known(id).is_none() || matches!(event.damage_type, "minecraft:out_of_world" | "minecraft:generic_kill") {
            return Flow::Continue;
        }
        let worn = goals::baubles(id);
        let Some(bauble) = worn
            .iter()
            .find(|b| matches!(&***b, "alice_dolls:ultramarine_orb_elixir" | "minecraft:totem_of_undying"))
            .cloned()
        else {
            return Flow::Continue;
        };
        let health = match &*bauble {
            "alice_dolls:ultramarine_orb_elixir" => doll_of(host, id).map_or(HEALTH, health_of),
            // Vanilla's totem leaves one health and regeneration; a doll, with no effects to take, gets a third.
            "minecraft:totem_of_undying" => doll_of(host, id).map_or(HEALTH, health_of) / 3.0,
            _ => return Flow::Continue,
        };
        let Some(dimension) = host.entity(id).and_then(|e| host.entity_level(e)).map(|l| l.dimension().to_owned()) else {
            return Flow::Continue;
        };
        if !host.set_mob_health(&dimension, id, health) {
            return Flow::Continue;
        }
        // The one used is taken out of its grid.
        let holder = Holder::Entity(id);
        let mut data = DOLL.get(host, holder).unwrap_or_default();
        if let Some(slot) = data.baubles.iter_mut().find(|s| s.as_ref().is_some_and(|s| s.item == bauble)) {
            *slot = None;
        }
        goals::set_baubles(id, worn_names(&data.baubles));
        let _ = DOLL.set(host, holder, data);
        if let Some(at) = host.entity(id).and_then(|e| host.entity_position(e)) {
            host.play_local_sound("minecraft:item.totem.use", at, 1.0, 1.0, 0);
        }
        let data = DOLL.get(host, Holder::Entity(id)).unwrap_or_default();
        say(host, id, &data, 1, "That was close!");
        Flow::Cancel
    }

    /// A doll item that carries an inventory lists it, as a shulker box does.
    fn tooltip(&self, _: &dyn Host, event: &mut TooltipEvent<'_>) {
        if doll_for_item(&event.stack.item).is_none() {
            return;
        }
        let held = event.stack.container();
        for (_, s) in held.iter().take(TOOLTIP_STACKS) {
            let line = format!("§7{} x{}", item_label(&s.item), s.count);
            event.lines.push((Text::legacy(&line), u32::MAX));
        }
        if held.len() > TOOLTIP_STACKS {
            let line = format!("§7§oand {} more...", held.len() - TOOLTIP_STACKS);
            event.lines.push((Text::legacy(&line), u32::MAX));
        }
    }

    /// Draws the doll's own model instead of the chicken's.
    fn mob_submit(&self, host: &dyn Host, event: &mut MobSubmitEvent<'_>) -> Flow {
        if event.mob.kind != KIND {
            return Flow::Continue;
        }
        if goals::on_mission(event.mob.id) {
            return Flow::Cancel;
        }
        // A doll on the head isn't drawn from the eyes it sits over: it would fill the view.
        if goals::riding(event.mob.id)
            && let Some(camera) = host.camera()
            && let Some(at) = host.entity(event.mob.id).and_then(|e| host.entity_position(e))
        {
            let d2: f64 = (0..3).map(|i| (camera.position[i] - at[i]).powi(2)).sum();
            if d2 < RIDE_HIDE_RANGE * RIDE_HIDE_RANGE {
                return Flow::Cancel;
            }
        }
        let name = event.mob.custom_name;
        let Some(doll) = name.and_then(doll_named) else {
            if name.is_some_and(|n| n.contains("Doll")) && first_time(&RENDER_SEEN, event.mob.id) {
                debug(host, &format!("render {}: named {name:?}, which is no doll name", event.mob.id));
            }
            return Flow::Continue;
        };
        // Without its texture a doll would be invisible: then it shows as the chicken it is.
        let drawn = match doll.look {
            model::Look::Base => model::draw_doll(host, event.mob, event.draw, doll.sprite, doll.held),
            model::Look::Boxes(m) => model::draw_model(host, event.mob, event.draw, doll.sprite, doll.held, m),
        };
        if first_time(&RENDER_SEEN, event.mob.id) {
            debug(host, &format!("render {} {:?}: drawn={drawn}", event.mob.id, doll.name));
            if !drawn {
                // Which sprites the atlas has: a quad with no area draws nothing to see.
                let flat = [[0.0f32; 3]; 4];
                for sprite in [
                    doll.sprite,
                    "alice_dolls:entity/shanghai",
                    "alice_dolls:item/shanghai_doll",
                    "lunatic:entity/character/alice_margatroid",
                    "entity/chicken/chicken_temperate",
                    "minecraft:entity/chicken/chicken_temperate",
                    "block/stone",
                ] {
                    let found = host.draw_quad(event.draw, sprite, flat, [0.0; 4], 0xFF_FF_FF);
                    debug(host, &format!("  atlas has {sprite}: {found}"));
                }
            }
        }
        if drawn { Flow::Cancel } else { Flow::Continue }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dolls_data_survives_saving() {
        let mut items = vec![None; INVENTORY];
        items[3] = Some(ItemStack::new("minecraft:torch", 12));
        let data = DollData {
            owner: u128::MAX - 41,
            mode: goals::RIDE,
            enabled: false,
            chat_level: 3,
            items,
            bound: 99 << 70 | 5,
            keep_rock: true,
            baubles: { let mut b = vec![None; goals::BAUBLE_SLOTS]; b[0] = Some(ItemStack::new("alice_dolls:mute_bauble", 1)); b },
            anchor: Some([10, -40, 7]),
            xp: 1234,
            favour: 4321,
            form_level: 3,
        };
        assert_eq!(decode(&encode(&data).unwrap()).unwrap(), data);
        let data = DollData { owner: 7 << 64, ..DollData::default() };
        assert_eq!(decode(&encode(&data).unwrap()).unwrap(), data);
    }

    #[test]
    fn grand_guignol_cooks_as_a_furnace_would() {
        assert_eq!(cooked("minecraft:beef"), Some("minecraft:cooked_beef"));
        assert_eq!(cooked("minecraft:raw_iron"), Some("minecraft:iron_ingot"));
        assert_eq!(cooked("minecraft:oak_log"), Some("minecraft:charcoal"));
        assert_eq!(cooked("minecraft:blue_terracotta"), Some("minecraft:blue_glazed_terracotta"));
        assert_eq!(cooked("minecraft:blue_glazed_terracotta"), None);
        assert_eq!(cooked("minecraft:cooked_beef"), None);
        assert_eq!(cooked("thkaguyamod:point_item"), None);
    }

    #[test]
    fn only_mobs_with_heads_leave_them() {
        assert_eq!(mob_head("minecraft:wither_skeleton"), Some("minecraft:wither_skeleton_skull"));
        assert_eq!(mob_head("minecraft:piglin"), Some("minecraft:piglin_head"));
        assert_eq!(mob_head("minecraft:spider"), None);
        assert_eq!(mob_head("minecraft:husk"), None);
    }

    #[test]
    fn point_and_power_items_are_worth_favour() {
        assert_eq!(point_favour("thkaguyamod:point_item"), Some(POINT_FAVOUR));
        assert_eq!(point_favour("thkaguyamod:full_power_item"), Some(40));
        assert_eq!(point_favour("thkaguyamod:laevateinn"), None);
        assert_eq!(point_favour("minecraft:paper"), None);
    }

    #[test]
    fn the_favour_tag_shows_level_and_progress() {
        assert_eq!(favour_tag("Shanghai Doll", 2, 125, 306), "Shanghai Doll  Lv 2 ■■■■□□□□□□ 40%");
        assert_eq!(favour_tag("Horai Doll", 0, 0, 100), "Horai Doll  Lv 0 □□□□□□□□□□ 0%");
    }

    #[test]
    fn aya_looks_where_the_camera_would() {
        let (yaw, pitch) = goals::yaw_pitch([0.0, 0.0, 0.0], [0.0, 0.0, 5.0]);
        assert!(yaw.abs() < 1e-4 && pitch.abs() < 1e-4);
        let (yaw, _) = goals::yaw_pitch([0.0, 0.0, 0.0], [-5.0, 0.0, 0.0]);
        assert!((yaw - 90.0).abs() < 1e-3);
        let (_, pitch) = goals::yaw_pitch([0.0, 0.0, 0.0], [0.0, -5.0, 5.0]);
        assert!((pitch - 45.0).abs() < 1e-3);
    }

    #[test]
    fn favour_levels_take_three_quarters_more_each_time() {
        assert_eq!(favour_step(0), 100);
        assert_eq!(favour_step(1), 175);
        assert_eq!(favour_step(2), 306);
        assert_eq!(favour_level(0), (0, 0, 100));
        assert_eq!(favour_level(99), (0, 99, 100));
        assert_eq!(favour_level(100), (1, 0, 175));
        assert_eq!(favour_level(100 + 175 + 306), (3, 0, 536));
    }

    #[test]
    fn each_level_adds_flat_then_a_tenth() {
        assert_eq!(goals::levelled_attack(5.0, 0), 5.0);
        assert!((goals::levelled_attack(5.0, 1) - 8.8).abs() < 1e-4);
        assert!((goals::levelled_attack(5.0, 2) - 12.98).abs() < 1e-4);
        assert!((goals::levelled_health(30.0, 1) - 38.5).abs() < 1e-4);
        assert!((goals::level_boost(2) - 1.21).abs() < 1e-9);
    }

    #[test]
    fn a_doll_items_favour_survives_being_picked_up() {
        let horai = DOLLS.iter().find(|d| d.name == goals::HORAI).unwrap();
        let k = Known {
            doll: horai,
            owner: 1,
            items: vec![None; horai.slots],
            baubles: vec![None; goals::BAUBLE_SLOTS],
            favour: 777,
            form_level: 3,
        };
        let item = &belongings(&k)[0];
        assert_eq!(favour_in(item), (777, 3));
        assert_eq!(items_in(item, horai.slots), vec![None; horai.slots]);
        assert_eq!(baubles_in(item), vec![None; goals::BAUBLE_SLOTS]);
    }

    #[test]
    fn horai_and_goliath_grow_into_their_next_forms() {
        assert_eq!(next_form(goals::HORAI), Some(goals::DIAMOND_HORAI));
        assert_eq!(next_form(goals::DIAMOND_HORAI), Some(goals::NETHERITE_HORAI));
        assert_eq!(next_form(goals::NETHERITE_HORAI), None);
        assert_eq!(next_form(goals::GOLIATH), Some(goals::NETHERITE_GOLIATH));
        assert_eq!(next_form(goals::SHANGHAI), Some(goals::GOLIATH));
        assert_eq!(next_form(goals::RUSSIA), Some(goals::MOSCOW));
        assert!(doll_named(goals::NETHERITE_GOLIATH).is_some());
    }

    #[test]
    fn dolls_eat_real_food_and_leave_harmful_food_alone() {
        assert_eq!(goals::food_heal("minecraft:cooked_beef"), Some(8.0));
        assert_eq!(goals::food_heal("minecraft:bread"), Some(5.0));
        assert_eq!(goals::food_heal("minecraft:rotten_flesh"), None);
        assert_eq!(goals::food_heal("minecraft:pufferfish"), None);
        assert_eq!(goals::food_heal("minecraft:torch"), None);
    }

    #[test]
    fn a_moscow_dolls_big_stacks_and_far_slots_survive_saving_and_her_item() {
        let mut items = vec![None; 54];
        items[0] = Some(ItemStack::new("minecraft:cobblestone", 128));
        items[40] = Some(ItemStack::new("minecraft:diamond_sword", 128));
        items[53] = Some(ItemStack::new("minecraft:dirt", 5));
        let data = DollData { items: items.clone(), ..DollData::default() };
        let back = decode(&encode(&data).unwrap()).unwrap();
        assert_eq!(back.items[..54], items[..]);
        let moscow = DOLLS.iter().find(|d| d.name == goals::MOSCOW).unwrap();
        assert_eq!(items_in(&doll_item(moscow, &items), 54), items);
    }

    #[test]
    fn a_doll_saved_by_the_first_version_keeps_its_mode() {
        let saved = |standby: i8| -> Compound {
            [
                ("owner_high".into(), Tag::Long(0)),
                ("owner_low".into(), Tag::Long(9)),
                ("standby".into(), Tag::Byte(standby)),
            ]
            .into()
        };
        assert_eq!(decode(&saved(1)).unwrap().mode, goals::STANDBY);
        let following = decode(&saved(0)).unwrap();
        assert_eq!(following.mode, goals::FOLLOW);
        assert!(following.enabled);
        assert_eq!(following.chat_level, DEFAULT_CHAT_LEVEL);
        assert_eq!(following.items, vec![None; INVENTORY]);
    }

    #[test]
    fn a_picked_up_doll_keeps_its_inventory_in_its_item() {
        let russia = doll_named(goals::RUSSIA).unwrap();
        let mut items = vec![None; russia.slots];
        items[0] = Some(ItemStack::new("minecraft:torch", 12));
        items[35] = Some(ItemStack::new("minecraft:iron_ingot", 3));
        let stack = doll_item(russia, &items);
        assert_eq!(stack.item.as_ref(), russia.item);
        assert_eq!(stack.container().len(), 2);
        assert_eq!(items_in(&stack, russia.slots), items);
        // An empty doll's item is a plain one, which stacks.
        assert!(doll_item(russia, &vec![None; russia.slots]).extra.is_none());
        // Into fewer slots, what is past them takes the empty ones.
        let fewer = items_in(&stack, 9);
        assert_eq!(fewer.iter().flatten().count(), 2);
    }

    #[test]
    fn a_tooltip_names_items_as_the_game_does() {
        assert_eq!(item_label("minecraft:iron_ingot"), "Iron Ingot");
        assert_eq!(item_label("torch"), "Torch");
    }

    #[test]
    fn what_does_not_fit_is_left_over() {
        let mut items = vec![Some(ItemStack::new("minecraft:torch", 60)), None];
        assert_eq!(room_for(&items, &ItemStack::new("minecraft:torch", 1), 64), 68);
        let left = store_some(&mut items, &ItemStack::new("minecraft:torch", 80), 64);
        assert_eq!(left, 12);
        assert_eq!(items[1].as_ref().map(|s| s.count), Some(64));
    }

    #[test]
    fn every_doll_has_its_own_name_and_item_and_whole_rows() {
        for (i, a) in DOLLS.iter().enumerate() {
            assert!(a.slots % 9 == 0 && a.slots <= 54, "{}", a.name);
            for b in &DOLLS[i + 1..] {
                assert_ne!(a.name, b.name);
                assert_ne!(a.item, b.item);
            }
        }
        assert_eq!(health_of(doll_named(goals::GOLIATH).unwrap()), 66.0);
        assert_eq!(health_of(doll_named(goals::NETHERITE_GOLIATH).unwrap()), 69.3);
        assert_eq!(health_of(doll_named(goals::REIMU).unwrap()), 30.0);
    }

    #[test]
    fn each_doll_is_found_by_its_item_and_its_name() {
        for doll in &DOLLS {
            assert_eq!(doll_for_item(doll.item), Some(doll));
            assert_eq!(doll_named(doll.name), Some(doll));
        }
        assert_eq!(doll_named("Shanghai"), None);
    }

    #[test]
    fn the_modes_go_round_in_the_originals_order() {
        assert_eq!(next_mode(goals::STANDBY), goals::PATROL);
        assert_eq!(next_mode(goals::PATROL), goals::FOLLOW);
        assert_eq!(next_mode(goals::FOLLOW), goals::RIDE);
        assert_eq!(next_mode(goals::RIDE), goals::STANDBY);
    }

    #[test]
    fn a_doll_is_summoned_quiet_named_and_with_its_health() {
        let nbt = doll_nbt(&DOLLS[1]);
        assert!(matches!(nbt.get(&"Silent".into()), Some(Tag::Byte(1))));
        assert!(matches!(nbt.get(&"CustomName".into()), Some(Tag::String(_))));
        assert!(matches!(nbt.get(&"Health".into()), Some(Tag::Float(b)) if *b == 30.0f32.to_bits()));
    }
}
