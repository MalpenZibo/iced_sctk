use iced_runtime::Command;

pub mod operation;

/// Focuses the previous focusable widget.
pub fn focus_previous<Message>() -> Command<Message>
where
    Message: 'static,
{
    Command::widget(iced_core::widget::operation::focusable::focus_previous())
}

/// Focuses the next focusable widget.
pub fn focus_next<Message>() -> Command<Message>
where
    Message: 'static,
{
    Command::widget(iced_core::widget::operation::focusable::focus_next())
}
