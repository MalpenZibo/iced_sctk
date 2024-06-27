use iced_core::window::Id as SurfaceId;
use iced_runtime::command::Command;

use crate::command::wayland;

pub fn request_token<Message: 'static>(
    app_id: Option<String>,
    window: Option<SurfaceId>,
    to_message: impl FnOnce(Option<String>) -> Message + Send + Sync + 'static,
) -> Command<Message> {
    Command::single(
        wayland::Action::Activation(wayland::activation::Action::RequestToken {
            app_id,
            window,
            message: Box::new(to_message),
        })
        .into(),
    )
}

pub fn activate<Message: 'static>(window: SurfaceId, token: String) -> Command<Message> {
    Command::single(
        wayland::Action::Activation(wayland::activation::Action::Activate { window, token }).into(),
    )
}
