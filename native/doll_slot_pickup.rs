// Before a menu's slot gives up its stack (`Slot.mayPickup`): the panes around a doll's bauble grid stay.
fn doll_slot_pickup(view: &crate::lan::menu::View<'_>, i: usize) -> core::ops::ControlFlow<bool> {
    let Some(window) = view.guest.menu.as_ref().map(|m| m.window) else {
        return core::ops::ControlFlow::Continue(());
    };
    match crate::mobs::alice_doll_goals::menu_may_pickup(view.guest.uuid, window, i) {
        Some(may) => core::ops::ControlFlow::Break(may),
        None => core::ops::ControlFlow::Continue(()),
    }
}
