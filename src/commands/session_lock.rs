use iced_core::window::Id as SurfaceId;
use iced_runtime::command::Command;
use sctk::reexports::client::protocol::wl_output::WlOutput;

use std::marker::PhantomData;

use crate::command::wayland;

pub fn lock<Message: 'static>() -> Command<Message> {
    Command::single(wayland::Action::SessionLock(wayland::session_lock::Action::Lock).into())
}

pub fn unlock<Message: 'static>() -> Command<Message> {
    Command::single(wayland::Action::SessionLock(wayland::session_lock::Action::Unlock).into())
}

pub fn get_lock_surface<Message: 'static>(id: SurfaceId, output: WlOutput) -> Command<Message> {
    Command::single(
        wayland::Action::SessionLock(wayland::session_lock::Action::LockSurface {
            id,
            output,
            _phantom: PhantomData,
        })
        .into(),
    )
}

pub fn destroy_lock_surface<Message: 'static>(id: SurfaceId) -> Command<Message> {
    Command::single(
        wayland::Action::SessionLock(wayland::session_lock::Action::DestroyLockSurface { id })
            .into(),
    )
}
