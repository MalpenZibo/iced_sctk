use crate::command::platform_specific::wayland;
use iced_core::window::Id as SurfaceId;
use iced_runtime::command::Command;
use iced_runtime::command::{self};

pub fn request_token<Message: 'static>(
    app_id: Option<String>,
    window: Option<SurfaceId>,
    to_message: impl FnOnce(Option<String>) -> Message + Send + Sync + 'static,
) -> Command<Message> {
    Command::single(command::Action::Custom(Box::new(
        wayland::Action::Activation::<Message>(wayland::activation::Action::RequestToken {
            app_id,
            window,
            message: Box::new(to_message),
        }),
    )))
}

pub fn activate<Message: 'static>(window: SurfaceId, token: String) -> Command<Message> {
    Command::single(command::Action::Custom(Box::new(
        wayland::Action::Activation::<Message>(wayland::activation::Action::Activate {
            window,
            token,
        }),
    )))
}
