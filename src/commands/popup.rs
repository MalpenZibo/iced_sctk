//! Interact with the popups of your application.
use iced_core::window::Id as SurfaceId;
use iced_runtime::command::Command;

use crate::command::wayland::{self, popup::SctkPopupSettings};

/// <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:request:get_popup>
/// <https://wayland.app/protocols/xdg-shell#xdg_surface:request:get_popup>
pub fn get_popup<Message: 'static>(popup: SctkPopupSettings) -> Command<Message> {
    Command::single(
        wayland::Action::Popup(wayland::popup::Action::Popup {
            popup,
            _phantom: Default::default(),
        })
        .into(),
    )
}

/// <https://wayland.app/protocols/xdg-shell#xdg_popup:request:reposition>
pub fn set_size<Message: 'static>(id: SurfaceId, width: u32, height: u32) -> Command<Message> {
    Command::single(
        wayland::Action::Popup(wayland::popup::Action::Size { id, width, height }).into(),
    )
}

// https://wayland.app/protocols/xdg-shell#xdg_popup:request:grab
pub fn grab_popup<Message: 'static>(id: SurfaceId) -> Command<Message> {
    Command::single(wayland::Action::Popup(wayland::popup::Action::Grab { id }).into())
}

/// <https://wayland.app/protocols/xdg-shell#xdg_popup:request:destroy>
pub fn destroy_popup<Message: 'static>(id: SurfaceId) -> Command<Message> {
    Command::single(wayland::Action::Popup(wayland::popup::Action::Destroy { id }).into())
}
