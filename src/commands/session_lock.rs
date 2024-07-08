use crate::command::platform_specific::wayland;
use iced_core::window::Id as SurfaceId;
use iced_runtime::command::Command;
use iced_runtime::command::{self};
use sctk::reexports::client::protocol::wl_output::WlOutput;

use std::marker::PhantomData;

pub fn lock<Message: 'static>() -> Command<Message> {
    Command::single(command::Action::Custom(Box::new(
        wayland::Action::SessionLock::<Message>(wayland::session_lock::Action::Lock),
    )))
}

pub fn unlock<Message: 'static>() -> Command<Message> {
    Command::single(command::Action::Custom(Box::new(
        wayland::Action::SessionLock::<Message>(wayland::session_lock::Action::Unlock),
    )))
}

pub fn get_lock_surface<Message: 'static>(id: SurfaceId, output: WlOutput) -> Command<Message> {
    Command::single(command::Action::Custom(Box::new(
        wayland::Action::SessionLock::<Message>(wayland::session_lock::Action::LockSurface {
            id,
            output,
            _phantom: PhantomData,
        }),
    )))
}

pub fn destroy_lock_surface<Message: 'static>(id: SurfaceId) -> Command<Message> {
    Command::single(command::Action::Custom(Box::new(
        wayland::Action::SessionLock::<Message>(
            wayland::session_lock::Action::DestroyLockSurface { id },
        ),
    )))
}
