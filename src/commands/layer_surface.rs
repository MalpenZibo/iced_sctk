//! Interact with the window of your application.
use std::marker::PhantomData;

use iced_core::window::Id as SurfaceId;

use iced_runtime::Command;
pub use sctk::shell::wlr_layer::{Anchor, KeyboardInteractivity, Layer};

use crate::command::wayland::{
    self,
    layer_surface::{IcedMargin, SctkLayerSurfaceSettings},
};

// TODO ASHLEY: maybe implement as builder that outputs a batched commands
/// <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_shell_v1:request:get_layer_surface>
pub fn get_layer_surface<Message: 'static>(builder: SctkLayerSurfaceSettings) -> Command<Message> {
    Command::single(
        wayland::Action::LayerSurface(wayland::layer_surface::Action::LayerSurface {
            builder,
            _phantom: PhantomData,
        })
        .into(),
    )
}

/// <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:request:destroy>
pub fn destroy_layer_surface<Message: 'static>(id: SurfaceId) -> Command<Message> {
    Command::single(
        wayland::Action::LayerSurface(wayland::layer_surface::Action::Destroy(id)).into(),
    )
}

/// <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:request:set_size>
pub fn set_size<Message: 'static>(
    id: SurfaceId,
    width: Option<u32>,
    height: Option<u32>,
) -> Command<Message> {
    Command::single(
        wayland::Action::LayerSurface(wayland::layer_surface::Action::Size { id, width, height })
            .into(),
    )
}
/// <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:request:set_anchor>
pub fn set_anchor<Message: 'static>(id: SurfaceId, anchor: Anchor) -> Command<Message> {
    Command::single(
        wayland::Action::LayerSurface(wayland::layer_surface::Action::Anchor { id, anchor }).into(),
    )
}
/// <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:request:set_exclusive_zone>
pub fn set_exclusive_zone<Message: 'static>(id: SurfaceId, zone: i32) -> Command<Message> {
    Command::single(
        wayland::Action::LayerSurface(wayland::layer_surface::Action::ExclusiveZone {
            id,
            exclusive_zone: zone,
        })
        .into(),
    )
}

/// <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:request:set_margin>
pub fn set_margin<Message: 'static>(
    id: SurfaceId,
    top: i32,
    right: i32,
    bottom: i32,
    left: i32,
) -> Command<Message> {
    Command::single(
        wayland::Action::LayerSurface(wayland::layer_surface::Action::Margin {
            id,
            margin: IcedMargin {
                top,
                right,
                bottom,
                left,
            },
        })
        .into(),
    )
}

/// <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:request:set_keyboard_interactivity>
pub fn set_keyboard_interactivity<Message: 'static>(
    id: SurfaceId,
    keyboard_interactivity: KeyboardInteractivity,
) -> Command<Message> {
    Command::single(
        wayland::Action::LayerSurface(wayland::layer_surface::Action::KeyboardInteractivity {
            id,
            keyboard_interactivity,
        })
        .into(),
    )
}

/// <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:request:set_layer>
pub fn set_layer<Message: 'static>(id: SurfaceId, layer: Layer) -> Command<Message> {
    Command::single(
        wayland::Action::LayerSurface(wayland::layer_surface::Action::Layer { id, layer }).into(),
    )
}
