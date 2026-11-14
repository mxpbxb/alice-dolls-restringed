// How many a menu's slot holds (`Slot.getMaxStackSize`): one piece in a doll's armour and shield
// slots, and 128 of anything in a Moscow doll's other slots.
fn doll_slot_max(
    view: &crate::lan::menu::View<'_>,
    i: usize,
) -> core::ops::ControlFlow<u32> {
    let Some(window) = view.guest.menu.as_ref().map(|m| m.window) else {
        return core::ops::ControlFlow::Continue(());
    };
    match crate::mobs::alice_doll_goals::menu_slot_max(view.guest.uuid, window, i) {
        Some(max) => core::ops::ControlFlow::Break(max),
        None => core::ops::ControlFlow::Continue(()),
    }
}
