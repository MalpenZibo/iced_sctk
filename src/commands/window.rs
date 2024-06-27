//! Interact with the window of your application.
use std::marker::PhantomData;

use iced_core::window;
use iced_runtime::{core::window::Mode, Command};

use crate::command::wayland::{self, window::SctkWindowSettings};

pub fn get_window<Message: 'static>(builder: SctkWindowSettings) -> Command<Message> {
    Command::single(
        wayland::Action::Window(wayland::window::Action::Window {
            builder,
            _phantom: PhantomData,
        })
        .into(),
    )
}

// TODO Ashley refactor to use regular window events maybe...
/// close the window
pub fn close_window<Message: 'static>(id: window::Id) -> Command<Message> {
    Command::single(wayland::Action::Window(wayland::window::Action::Destroy(id)).into())
}

/// Resizes the window to the given logical dimensions.
pub fn resize_window<Message: 'static>(
    id: window::Id,
    width: u32,
    height: u32,
) -> Command<Message> {
    Command::single(
        wayland::Action::Window(wayland::window::Action::Size { id, width, height }).into(),
    )
}

pub fn start_drag_window<Message: 'static>(id: window::Id) -> Command<Message> {
    Command::single(wayland::Action::Window(wayland::window::Action::InteractiveMove { id }).into())
}

pub fn maximize<Message: 'static>(id: window::Id, maximized: bool) -> Command<Message> {
    Command::single(
        wayland::Action::Window(if maximized {
            wayland::window::Action::Maximize { id }
        } else {
            wayland::window::Action::UnsetMaximize { id }
        })
        .into(),
    )
}

pub fn toggle_maximize<Message: 'static>(id: window::Id) -> Command<Message> {
    Command::single(wayland::Action::Window(wayland::window::Action::ToggleMaximized { id }).into())
}

pub fn set_app_id_window<Message: 'static>(id: window::Id, app_id: String) -> Command<Message> {
    Command::single(wayland::Action::Window(wayland::window::Action::AppId { id, app_id }).into())
}

/// Sets the [`Mode`] of the window.
pub fn set_mode_window<Message: 'static>(id: window::Id, mode: Mode) -> Command<Message> {
    Command::single(wayland::Action::Window(wayland::window::Action::Mode(id, mode)).into())
}
