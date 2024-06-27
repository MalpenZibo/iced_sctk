//! Query or update internal widget state.
pub mod search_id;

use iced_core::widget::operation::{Focusable, Outcome, Scrollable, TextInput};
use iced_core::widget::{Id, Operation};
use iced_core::{Rectangle, Vector};

use std::any::Any;

#[allow(missing_debug_implementations)]
/// A wrapper around an [`Operation`] that can be used for Application Messages and internally in Iced.
pub enum OperationWrapper<M> {
    /// Application Message
    Message(Box<dyn Operation<M>>),
    /// Widget Id
    Id(Box<dyn Operation<Id>>),
    /// Wrapper
    Wrapper(Box<dyn Operation<OperationOutputWrapper<M>>>),
}

#[allow(missing_debug_implementations)]
/// A wrapper around an [`Operation`] output that can be used for Application Messages and internally in Iced.
pub enum OperationOutputWrapper<M> {
    /// Application Message
    Message(M),
    /// Widget Id
    Id(Id),
}

impl<M: 'static> Operation<OperationOutputWrapper<M>> for OperationWrapper<M> {
    fn container(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        operate_on_children: &mut dyn FnMut(&mut dyn Operation<OperationOutputWrapper<M>>),
    ) {
        match self {
            OperationWrapper::Message(operation) => {
                operation.container(id, bounds, &mut |operation| {
                    operate_on_children(&mut MapOperation { operation });
                });
            }
            OperationWrapper::Id(operation) => {
                operation.container(id, bounds, &mut |operation| {
                    operate_on_children(&mut MapOperation { operation });
                });
            }
            OperationWrapper::Wrapper(operation) => {
                operation.container(id, bounds, operate_on_children);
            }
        }
    }

    fn focusable(&mut self, state: &mut dyn Focusable, id: Option<&Id>) {
        match self {
            OperationWrapper::Message(operation) => {
                operation.focusable(state, id);
            }
            OperationWrapper::Id(operation) => {
                operation.focusable(state, id);
            }
            OperationWrapper::Wrapper(operation) => {
                operation.focusable(state, id);
            }
        }
    }

    fn scrollable(
        &mut self,
        state: &mut dyn Scrollable,
        id: Option<&Id>,
        bounds: Rectangle,
        translation: Vector,
    ) {
        match self {
            OperationWrapper::Message(operation) => {
                operation.scrollable(state, id, bounds, translation);
            }
            OperationWrapper::Id(operation) => {
                operation.scrollable(state, id, bounds, translation);
            }
            OperationWrapper::Wrapper(operation) => {
                operation.scrollable(state, id, bounds, translation);
            }
        }
    }

    fn text_input(&mut self, state: &mut dyn TextInput, id: Option<&Id>) {
        match self {
            OperationWrapper::Message(operation) => {
                operation.text_input(state, id);
            }
            OperationWrapper::Id(operation) => {
                operation.text_input(state, id);
            }
            OperationWrapper::Wrapper(operation) => {
                operation.text_input(state, id);
            }
        }
    }

    fn finish(&self) -> Outcome<OperationOutputWrapper<M>> {
        match self {
            OperationWrapper::Message(operation) => match operation.finish() {
                Outcome::None => Outcome::None,
                Outcome::Some(o) => Outcome::Some(OperationOutputWrapper::Message(o)),
                Outcome::Chain(c) => Outcome::Chain(Box::new(OperationWrapper::Message(c))),
            },
            OperationWrapper::Id(operation) => match operation.finish() {
                Outcome::None => Outcome::None,
                Outcome::Some(id) => Outcome::Some(OperationOutputWrapper::Id(id)),
                Outcome::Chain(c) => Outcome::Chain(Box::new(OperationWrapper::Id(c))),
            },
            OperationWrapper::Wrapper(c) => c.as_ref().finish(),
        }
    }

    fn custom(&mut self, _state: &mut dyn Any, _id: Option<&Id>) {
        match self {
            OperationWrapper::Message(operation) => {
                operation.custom(_state, _id);
            }
            OperationWrapper::Id(operation) => {
                operation.custom(_state, _id);
            }
            OperationWrapper::Wrapper(operation) => {
                operation.custom(_state, _id);
            }
        }
    }
}

#[allow(missing_debug_implementations)]
/// Map Operation
pub struct MapOperation<'a, B> {
    /// inner operation
    pub(crate) operation: &'a mut dyn Operation<B>,
}

impl<'a, B> MapOperation<'a, B> {
    /// Creates a new [`MapOperation`].
    pub fn new(operation: &'a mut dyn Operation<B>) -> MapOperation<'a, B> {
        MapOperation { operation }
    }
}

impl<'a, T, B> Operation<T> for MapOperation<'a, B> {
    fn container(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        operate_on_children: &mut dyn FnMut(&mut dyn Operation<T>),
    ) {
        self.operation.container(id, bounds, &mut |operation| {
            operate_on_children(&mut MapOperation { operation });
        });
    }

    fn focusable(&mut self, state: &mut dyn Focusable, id: Option<&Id>) {
        self.operation.focusable(state, id);
    }

    fn scrollable(
        &mut self,
        state: &mut dyn Scrollable,
        id: Option<&Id>,
        bounds: Rectangle,
        translation: Vector,
    ) {
        self.operation.scrollable(state, id, bounds, translation);
    }

    fn text_input(&mut self, state: &mut dyn TextInput, id: Option<&Id>) {
        self.operation.text_input(state, id)
    }

    fn custom(&mut self, state: &mut dyn Any, id: Option<&Id>) {
        self.operation.custom(state, id);
    }
}

/// Produces an [`Operation`] that applies the given [`Operation`] to the
/// children of a container with the given [`Id`].
pub fn scoped<T: 'static>(target: Id, operation: impl Operation<T> + 'static) -> impl Operation<T> {
    struct ScopedOperation<Message> {
        target: Id,
        operation: Box<dyn Operation<Message>>,
    }

    impl<Message: 'static> Operation<Message> for ScopedOperation<Message> {
        fn container(
            &mut self,
            id: Option<&Id>,
            _bounds: Rectangle,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<Message>),
        ) {
            if id == Some(&self.target) {
                operate_on_children(self.operation.as_mut());
            } else {
                operate_on_children(self);
            }
        }

        fn finish(&self) -> Outcome<Message> {
            match self.operation.finish() {
                Outcome::Chain(next) => Outcome::Chain(Box::new(ScopedOperation {
                    target: self.target.clone(),
                    operation: next,
                })),
                outcome => outcome,
            }
        }
    }

    ScopedOperation {
        target,
        operation: Box::new(operation),
    }
}
