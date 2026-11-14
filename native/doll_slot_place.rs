// Before a menu's slot takes a stack (`Slot.mayPlace`): in a doll's inventory screen its armour slots
// take only their piece, and its shield slot only a shield, as Touhou Little Maid's maid screen does.
fn doll_slot_place(
    view: &crate::lan::menu::View<'_>,
    i: usize,
    stack: &crate::items::ItemStack,
) -> core::ops::ControlFlow<bool> {
    let Some(window) = view.guest.menu.as_ref().map(|m| m.window) else {
        return core::ops::ControlFlow::Continue(());
    };
    match crate::mobs::alice_doll_goals::menu_may_place(view.guest.uuid, window, i, &stack.item) {
        Some(may) => core::ops::ControlFlow::Break(may),
        None => core::ops::ControlFlow::Continue(()),
    }
}
