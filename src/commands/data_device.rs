//! Interact with the data device objects of your application.
use crate::command::platform_specific::wayland::{
    self,
    data_device::{ActionInner, DataFromMimeType, DndIcon},
};
use iced_core::window;
use iced_runtime::{
    command::{self},
    Command,
};
use sctk::reexports::client::protocol::wl_data_device_manager::DndAction;

use crate::core::Vector;

/// start an internal drag and drop operation. Events will only be delivered to the same client.
/// The client is responsible for data transfer.
pub fn start_internal_drag<Message: 'static>(
    origin_id: window::Id,
    icon_id: Option<window::Id>,
) -> Command<Message> {
    Command::single(command::Action::Custom(Box::new(
        wayland::Action::DataDevice::<Message>(
            wayland::data_device::ActionInner::StartInternalDnd { origin_id, icon_id }.into(),
        ),
    )))
}

/// Start a drag and drop operation. When a client asks for the selection, an event will be delivered
/// to the client with the fd to write the data to.
pub fn start_drag<Message: 'static>(
    mime_types: Vec<String>,
    actions: DndAction,
    origin_id: window::Id,
    icon_id: Option<(DndIcon, Vector)>,
    data: Box<dyn DataFromMimeType + Send + Sync>,
) -> Command<Message> {
    Command::single(command::Action::Custom(Box::new(
        wayland::Action::DataDevice::<Message>(
            wayland::data_device::ActionInner::StartDnd {
                mime_types,
                actions,
                origin_id,
                icon_id,
                data,
            }
            .into(),
        ),
    )))
}

/// Set accepted and preferred drag and drop actions.
pub fn set_actions<Message: 'static>(
    preferred: DndAction,
    accepted: DndAction,
) -> Command<Message> {
    Command::single(command::Action::Custom(Box::new(
        wayland::Action::DataDevice::<Message>(
            wayland::data_device::ActionInner::SetActions {
                preferred,
                accepted,
            }
            .into(),
        ),
    )))
}

/// Accept a mime type or None to reject the drag and drop operation.
pub fn accept_mime_type<Message: 'static>(mime_type: Option<String>) -> Command<Message> {
    Command::single(command::Action::Custom(Box::new(
        wayland::Action::DataDevice::<Message>(
            wayland::data_device::ActionInner::Accept(mime_type).into(),
        ),
    )))
}

/// Read drag and drop data. This will trigger an event with the data.
pub fn request_dnd_data<Message: 'static>(mime_type: String) -> Command<Message> {
    Command::single(command::Action::Custom(Box::new(
        wayland::Action::DataDevice::<Message>(
            wayland::data_device::ActionInner::RequestDndData(mime_type).into(),
        ),
    )))
}

/// Finished the drag and drop operation.
pub fn finish_dnd<Message: 'static>() -> Command<Message> {
    Command::single(command::Action::Custom(Box::new(
        wayland::Action::DataDevice::<Message>(
            wayland::data_device::ActionInner::DndFinished.into(),
        ),
    )))
}

/// Cancel the drag and drop operation.
pub fn cancel_dnd<Message: 'static>() -> Command<Message> {
    Command::single(command::Action::Custom(Box::new(
        wayland::Action::DataDevice::<Message>(
            wayland::data_device::ActionInner::DndCancelled.into(),
        ),
    )))
}

/// Run a generic drag action
pub fn action<Message: 'static>(action: ActionInner) -> Command<Message> {
    Command::single(command::Action::Custom(Box::new(
        wayland::Action::DataDevice::<Message>(action.into()),
    )))
}
