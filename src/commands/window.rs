//! Interact with the window of your application.
use crate::command::platform_specific::wayland::{self, window::SctkWindowSettings};
use iced_core::window;
use iced_runtime::command::{self};
use iced_runtime::{core::window::Mode, Command};
use std::marker::PhantomData;

pub fn get_window<Message: 'static>(builder: SctkWindowSettings) -> Command<Message> {
    Command::single(command::Action::Custom(Box::new(
        wayland::Action::Window::<Message>(wayland::window::Action::Window {
            builder,
            _phantom: PhantomData,
        }),
    )))
}

// TODO Ashley refactor to use regular window events maybe...
/// close the window
pub fn close_window<Message: 'static>(id: window::Id) -> Command<Message> {
    Command::single(command::Action::Custom(Box::new(
        wayland::Action::Window::<Message>(wayland::window::Action::Destroy(id)),
    )))
}

/// Resizes the window to the given logical dimensions.
pub fn resize_window<Message: 'static>(
    id: window::Id,
    width: u32,
    height: u32,
) -> Command<Message> {
    Command::single(command::Action::Custom(Box::new(
        wayland::Action::Window::<Message>(wayland::window::Action::Size { id, width, height }),
    )))
}

pub fn start_drag_window<Message: 'static>(id: window::Id) -> Command<Message> {
    Command::single(command::Action::Custom(Box::new(
        wayland::Action::Window::<Message>(wayland::window::Action::InteractiveMove { id }),
    )))
}

pub fn maximize<Message: 'static>(id: window::Id, maximized: bool) -> Command<Message> {
    Command::single(command::Action::Custom(Box::new(
        wayland::Action::Window::<Message>(if maximized {
            wayland::window::Action::Maximize { id }
        } else {
            wayland::window::Action::UnsetMaximize { id }
        }),
    )))
}

pub fn toggle_maximize<Message: 'static>(id: window::Id) -> Command<Message> {
    Command::single(command::Action::Custom(Box::new(
        wayland::Action::Window::<Message>(wayland::window::Action::ToggleMaximized { id }),
    )))
}

pub fn set_app_id_window<Message: 'static>(id: window::Id, app_id: String) -> Command<Message> {
    Command::single(command::Action::Custom(Box::new(
        wayland::Action::Window::<Message>(wayland::window::Action::AppId { id, app_id }),
    )))
}

/// Sets the [`Mode`] of the window.
pub fn set_mode_window<Message: 'static>(id: window::Id, mode: Mode) -> Command<Message> {
    Command::single(command::Action::Custom(Box::new(
        wayland::Action::Window::<Message>(wayland::window::Action::Mode(id, mode)),
    )))
}
