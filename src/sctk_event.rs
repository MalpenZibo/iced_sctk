use crate::{
    application::SurfaceIdWrapper,
    command::platform_specific::wayland::data_device::DndIcon,
    conversion::{modifiers_to_native, pointer_axis_to_native, pointer_button_to_native},
    dpi::PhysicalSize,
    keymap::{self, keysym_to_key},
    subsurface_widget::SubsurfaceState,
};

use iced_core::window::Id as SurfaceId;
use iced_runtime::{
    // command::platform_specific::wayland::data_device::DndIcon,
    core::{/* event::wayland, */ keyboard, mouse, touch, window, Point},
    keyboard::{key, Key, Location},
};
use sctk::{
    output::OutputInfo,
    reexports::client::{
        backend::ObjectId,
        protocol::{
            wl_data_device_manager::DndAction, wl_keyboard::WlKeyboard, wl_output::WlOutput,
            wl_pointer::WlPointer, wl_seat::WlSeat, wl_surface::WlSurface, wl_touch::WlTouch,
        },
        Proxy,
    },
    reexports::csd_frame::WindowManagerCapabilities,
    seat::{
        keyboard::{KeyEvent, Modifiers},
        pointer::{PointerEvent, PointerEventKind},
        Capability,
    },
    session_lock::SessionLockSurfaceConfigure,
    shell::{
        wlr_layer::LayerSurfaceConfigure,
        xdg::{popup::PopupConfigure, window::WindowConfigure},
    },
};
use std::{collections::HashMap, num::NonZeroU32, time::Instant};
use wayland_protocols::wp::viewporter::client::wp_viewport::WpViewport;
use xkeysym::Keysym;

pub enum IcedSctkEvent<T> {
    /// Emitted when new events arrive from the OS to be processed.
    ///
    /// This event type is useful as a place to put code that should be done before you start
    /// processing events, such as updating frame timing information for benchmarking or checking
    /// the [`StartCause`][crate::event::StartCause] to see if a timer set by
    /// [`ControlFlow::WaitUntil`](crate::event_loop::ControlFlow::WaitUntil) has elapsed.
    NewEvents(StartCause),

    /// Any user event from iced
    UserEvent(T),

    /// An event produced by sctk
    SctkEvent(SctkEvent),

    #[cfg(feature = "a11y")]
    A11ySurfaceCreated(
        SurfaceIdWrapper,
        crate::event_loop::adapter::IcedSctkAdapter,
    ),

    /// emitted after first accessibility tree is requested
    #[cfg(feature = "a11y")]
    A11yEnabled,

    /// accessibility event
    #[cfg(feature = "a11y")]
    A11yEvent(ActionRequestEvent),

    /// Emitted when all of the event loop's input events have been processed and redraw processing
    /// is about to begin.
    ///
    /// This event is useful as a place to put your code that should be run after all
    /// state-changing events have been handled and you want to do stuff (updating state, performing
    /// calculations, etc) that happens as the "main body" of your event loop. If your program only draws
    /// graphics when something changes, it's usually better to do it in response to
    /// [`Event::RedrawRequested`](crate::event::Event::RedrawRequested), which gets emitted
    /// immediately after this event. Programs that draw graphics continuously, like most games,
    /// can render here unconditionally for simplicity.
    MainEventsCleared,

    /// Emitted after [`MainEventsCleared`] when a window should be redrawn.
    ///
    /// This gets triggered in two scenarios:
    /// - The OS has performed an operation that's invalidated the window's contents (such as
    ///   resizing the window).
    /// - The application has explicitly requested a redraw via [`Window::request_redraw`].
    ///
    /// During each iteration of the event loop, Winit will aggregate duplicate redraw requests
    /// into a single event, to help avoid duplicating rendering work.
    ///
    /// Mainly of interest to applications with mostly-static graphics that avoid redrawing unless
    /// something changes, like most non-game GUIs.
    ///
    /// [`MainEventsCleared`]: Self::MainEventsCleared
    RedrawRequested(ObjectId),

    /// Emitted after all [`RedrawRequested`] events have been processed and control flow is about to
    /// be taken away from the program. If there are no `RedrawRequested` events, it is emitted
    /// immediately after `MainEventsCleared`.
    ///
    /// This event is useful for doing any cleanup or bookkeeping work after all the rendering
    /// tasks have been completed.
    ///
    /// [`RedrawRequested`]: Self::RedrawRequested
    RedrawEventsCleared,

    /// Emitted when the event loop is being shut down.
    ///
    /// This is irreversible - if this event is emitted, it is guaranteed to be the last event that
    /// gets emitted. You generally want to treat this as an "do on quit" event.
    LoopDestroyed,

    /// Dnd source created with an icon surface.
    DndSurfaceCreated(WlSurface, DndIcon, SurfaceId),

    /// Frame callback event
    Frame(WlSurface, u32),

    Subcompositor(SubsurfaceState<T>),
}

#[derive(Debug, Clone)]
pub enum SctkEvent {
    //
    // Input events
    //
    SeatEvent {
        variant: SeatEventVariant,
        id: WlSeat,
    },
    PointerEvent {
        variant: PointerEvent,
        ptr_id: WlPointer,
        seat_id: WlSeat,
    },
    KeyboardEvent {
        variant: KeyboardEventVariant,
        kbd_id: WlKeyboard,
        seat_id: WlSeat,
    },
    TouchEvent {
        variant: touch::Event,
        touch_id: WlTouch,
        seat_id: WlSeat,
        surface: WlSurface,
    },
    // TODO data device & touch

    //
    // Surface Events
    //
    WindowEvent {
        variant: WindowEventVariant,
        id: WlSurface,
    },
    LayerSurfaceEvent {
        variant: LayerSurfaceEventVariant,
        id: WlSurface,
    },
    PopupEvent {
        variant: PopupEventVariant,
        /// this may be the Id of a window or layer surface
        toplevel_id: WlSurface,
        /// this may be any SurfaceId
        parent_id: WlSurface,
        /// the id of this popup
        id: WlSurface,
    },

    //
    // output events
    //
    NewOutput {
        id: WlOutput,
        info: Option<OutputInfo>,
    },
    UpdateOutput {
        id: WlOutput,
        info: OutputInfo,
    },
    RemovedOutput(WlOutput),
    //
    // compositor events
    //
    ScaleFactorChanged {
        factor: f64,
        id: WlOutput,
        inner_size: PhysicalSize<u32>,
    },
    DataSource(DataSourceEvent),
    DndOffer {
        event: DndOfferEvent,
        surface: WlSurface,
    },
    /// session lock events
    SessionLocked,
    SessionLockFinished,
    SessionLockSurfaceCreated {
        surface: WlSurface,
        native_id: SurfaceId,
    },
    SessionLockSurfaceConfigure {
        surface: WlSurface,
        configure: SessionLockSurfaceConfigure,
        first: bool,
    },
    SessionLockSurfaceDone {
        surface: WlSurface,
    },
    SessionLockSurfaceScaleFactorChanged {
        surface: WlSurface,
        scale_factor: f64,
        viewport: Option<WpViewport>,
    },
    SessionUnlocked,
}

#[derive(Debug, Clone)]
pub enum DataSourceEvent {
    /// A DnD action has been accepted by the compositor for your source.
    DndActionAccepted(DndAction),
    /// A DnD mime type has been accepted by a client for your source.
    MimeAccepted(Option<String>),
    /// Dnd Finished event.
    DndFinished,
    /// Dnd Cancelled event.
    DndCancelled,
    /// Dnd Drop performed event.
    DndDropPerformed,
    /// Send the selection data to the clipboard.
    SendSelectionData {
        /// The mime type of the data to be sent
        mime_type: String,
    },
    /// Send the DnD data to the destination.
    SendDndData {
        /// The mime type of the data to be sent
        mime_type: String,
    },
}

#[derive(Debug, Clone)]
pub enum DndOfferEvent {
    /// A DnD offer has been introduced with the given mime types.
    Enter {
        x: f64,
        y: f64,
        mime_types: Vec<String>,
    },
    /// The DnD device has left.
    Leave,
    /// Drag and Drop Motion event.
    Motion {
        /// x coordinate of the pointer
        x: f64,
        /// y coordinate of the pointer
        y: f64,
    },
    /// A drop has been performed.
    DropPerformed,
    /// Read the DnD data
    Data {
        /// The raw data
        data: Vec<u8>,
        /// mime type of the data to read
        mime_type: String,
    },
    SourceActions(DndAction),
    SelectedAction(DndAction),
}

#[cfg(feature = "a11y")]
#[derive(Debug, Clone)]
pub struct ActionRequestEvent {
    pub surface_id: ObjectId,
    pub request: iced_accessibility::accesskit::ActionRequest,
}

#[derive(Debug, Clone)]
pub enum SeatEventVariant {
    New,
    Remove,
    NewCapability(Capability, ObjectId),
    RemoveCapability(Capability, ObjectId),
}

#[derive(Debug, Clone)]
pub enum KeyboardEventVariant {
    Leave(WlSurface),
    Enter(WlSurface),
    Press(KeyEvent),
    Repeat(KeyEvent),
    Release(KeyEvent),
    Modifiers(Modifiers),
}

#[derive(Debug, Clone)]
pub enum WindowEventVariant {
    Created(ObjectId, SurfaceId),
    /// <https://wayland.app/protocols/xdg-shell#xdg_toplevel:event:close>
    Close,
    /// <https://wayland.app/protocols/xdg-shell#xdg_toplevel:event:wm_capabilities>
    WmCapabilities(WindowManagerCapabilities),
    /// <https://wayland.app/protocols/xdg-shell#xdg_toplevel:event:configure_bounds>
    ConfigureBounds {
        width: u32,
        height: u32,
    },
    Configure((NonZeroU32, NonZeroU32), WindowConfigure, WlSurface, bool),
    Size((NonZeroU32, NonZeroU32), WlSurface, bool),
    /// window state changed
    StateChanged(sctk::reexports::csd_frame::WindowState),
    /// Scale Factor
    ScaleFactorChanged(f64, Option<WpViewport>),
}

#[derive(Debug, Clone)]
pub enum PopupEventVariant {
    /// Popup Created
    Created(ObjectId, SurfaceId),
    /// <https://wayland.app/protocols/xdg-shell#xdg_popup:event:popup_done>
    Done,
    /// <https://wayland.app/protocols/xdg-shell#xdg_popup:event:configure>
    Configure(PopupConfigure, WlSurface, bool),
    /// <https://wayland.app/protocols/xdg-shell#xdg_popup:event:repositioned>
    RepositionionedPopup { token: u32 },
    /// size
    Size(u32, u32),
    /// Scale Factor
    ScaleFactorChanged(f64, Option<WpViewport>),
}

#[derive(Debug, Clone)]
pub enum LayerSurfaceEventVariant {
    /// sent after creation of the layer surface
    Created(ObjectId, SurfaceId),
    /// <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:event:closed>
    Done,
    /// <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:event:configure>
    Configure(LayerSurfaceConfigure, WlSurface, bool),
    /// Scale Factor
    ScaleFactorChanged(f64, Option<WpViewport>),
}

/// Describes the reason the event loop is resuming.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartCause {
    /// Sent if the time specified by [`ControlFlow::WaitUntil`] has been reached. Contains the
    /// moment the timeout was requested and the requested resume time. The actual resume time is
    /// guaranteed to be equal to or after the requested resume time.
    ///
    /// [`ControlFlow::WaitUntil`]: crate::event_loop::ControlFlow::WaitUntil
    ResumeTimeReached {
        start: Instant,
        requested_resume: Instant,
    },

    /// Sent if the OS has new events to send to the window, after a wait was requested. Contains
    /// the moment the wait was requested and the resume time, if requested.
    WaitCancelled {
        start: Instant,
        requested_resume: Option<Instant>,
    },

    /// Sent if the event loop is being resumed after the loop's control flow was set to
    /// [`ControlFlow::Poll`].
    ///
    /// [`ControlFlow::Poll`]: crate::event_loop::ControlFlow::Poll
    Poll,

    /// Sent once, immediately after `run` is called. Indicates that the loop was just initialized.
    Init,
}

/// Pending update to a window requested by the user.
#[derive(Default, Debug, Clone, Copy)]
pub struct SurfaceUserRequest {
    /// Whether `redraw` was requested.
    pub redraw_requested: bool,

    /// Wether the frame should be refreshed.
    pub refresh_frame: bool,
}

// The window update coming from the compositor.
#[derive(Default, Debug, Clone)]
pub struct SurfaceCompositorUpdate {
    /// New window configure.
    pub configure: Option<WindowConfigure>,

    /// New scale factor.
    pub scale_factor: Option<i32>,
}

impl SctkEvent {
    pub fn to_native(
        self,
        modifiers: &mut Modifiers,
        surface_ids: &HashMap<ObjectId, SurfaceIdWrapper>,
        destroyed_surface_ids: &HashMap<ObjectId, SurfaceIdWrapper>,
        subsurface_ids: &HashMap<ObjectId, (i32, i32, SurfaceIdWrapper)>,
    ) -> Vec<iced_runtime::core::Event> {
        match self {
            // TODO Ashley: Platform specific multi-seat events?
            SctkEvent::SeatEvent { .. } => Default::default(),
            SctkEvent::PointerEvent { variant, .. } => match variant.kind {
                PointerEventKind::Enter { .. } => {
                    vec![iced_runtime::core::Event::Mouse(
                        mouse::Event::CursorEntered,
                    )]
                }
                PointerEventKind::Leave { .. } => {
                    vec![iced_runtime::core::Event::Mouse(mouse::Event::CursorLeft)]
                }
                PointerEventKind::Motion { .. } => {
                    let offset = if let Some((x_offset, y_offset, _)) =
                        subsurface_ids.get(&variant.surface.id())
                    {
                        (*x_offset, *y_offset)
                    } else {
                        (0, 0)
                    };
                    vec![iced_runtime::core::Event::Mouse(
                        mouse::Event::CursorMoved {
                            position: Point::new(
                                variant.position.0 as f32 + offset.0 as f32,
                                variant.position.1 as f32 + offset.1 as f32,
                            ),
                        },
                    )]
                }
                PointerEventKind::Press {
                    time: _,
                    button,
                    serial: _,
                } => pointer_button_to_native(button)
                    .map(|b| iced_runtime::core::Event::Mouse(mouse::Event::ButtonPressed(b)))
                    .into_iter()
                    .collect(), // TODO Ashley: conversion
                PointerEventKind::Release {
                    time: _,
                    button,
                    serial: _,
                } => pointer_button_to_native(button)
                    .map(|b| iced_runtime::core::Event::Mouse(mouse::Event::ButtonReleased(b)))
                    .into_iter()
                    .collect(), // TODO Ashley: conversion
                PointerEventKind::Axis {
                    time: _,
                    horizontal,
                    vertical,
                    source,
                } => pointer_axis_to_native(source, horizontal, vertical)
                    .map(|a| {
                        iced_runtime::core::Event::Mouse(mouse::Event::WheelScrolled { delta: a })
                    })
                    .into_iter()
                    .collect(), // TODO Ashley: conversion
            },
            SctkEvent::KeyboardEvent {
                variant,
                kbd_id: _,
                seat_id,
            } => {
                match variant {
                    KeyboardEventVariant::Leave(surface) => surface_ids
                        .get(&surface.id())
                        .and_then(|id| match id {
                            SurfaceIdWrapper::LayerSurface(_id) => {
                                // Some(iced_runtime::core::Event::PlatformSpecific(
                                //     PlatformSpecific::Wayland(
                                //         wayland::Event::Layer(
                                //             LayerEvent::Unfocused,
                                //             surface,
                                //             id.inner(),
                                //         ),
                                //     ),
                                // ))
                                None
                            }
                            SurfaceIdWrapper::Window(id) => Some(
                                iced_runtime::core::Event::Window(*id, window::Event::Unfocused),
                            ),
                            SurfaceIdWrapper::Popup(_id) => {
                                // Some(iced_runtime::core::Event::PlatformSpecific(
                                //     PlatformSpecific::Wayland(
                                //         wayland::Event::Popup(
                                //             PopupEvent::Unfocused,
                                //             surface,
                                //             id.inner(),
                                //         ),
                                //     ),
                                // ))
                                None
                            }
                            SurfaceIdWrapper::Dnd(_) => None,
                            SurfaceIdWrapper::SessionLock(_) => {
                                // Some(iced_runtime::core::Event::PlatformSpecific(
                                //     PlatformSpecific::Wayland(
                                //         wayland::Event::SessionLock(
                                //             SessionLockEvent::Unfocused(
                                //                 surface,
                                //                 id.inner(),
                                //             ),
                                //         ),
                                //     ),
                                // ))
                                None
                            }
                        })
                        .into_iter()
                        // .chain([iced_runtime::core::Event::PlatformSpecific(
                        //     PlatformSpecific::Wayland(wayland::Event::Seat(
                        //         wayland::SeatEvent::Leave,
                        //         seat_id,
                        //     )),
                        // )])
                        .collect(),
                    KeyboardEventVariant::Enter(surface) => surface_ids
                        .get(&surface.id())
                        .and_then(|id| match id {
                            SurfaceIdWrapper::LayerSurface(_id) => {
                                // Some(iced_runtime::core::Event::PlatformSpecific(
                                //     PlatformSpecific::Wayland(
                                //         wayland::Event::Layer(
                                //             LayerEvent::Focused,
                                //             surface,
                                //             id.inner(),
                                //         ),
                                //     ),
                                // ))
                                None
                            }
                            SurfaceIdWrapper::Window(id) => Some(
                                iced_runtime::core::Event::Window(*id, window::Event::Focused),
                            ),
                            SurfaceIdWrapper::Popup(_id) => {
                                // Some(iced_runtime::core::Event::PlatformSpecific(
                                //     PlatformSpecific::Wayland(
                                //         wayland::Event::Popup(
                                //             PopupEvent::Focused,
                                //             surface,
                                //             id.inner(),
                                //         ),
                                //     ),
                                // ))
                                None
                            }
                            SurfaceIdWrapper::Dnd(_) => None,
                            SurfaceIdWrapper::SessionLock(_) => {
                                // Some(iced_runtime::core::Event::PlatformSpecific(
                                //     PlatformSpecific::Wayland(
                                //         wayland::Event::SessionLock(
                                //             SessionLockEvent::Focused(
                                //                 surface,
                                //                 id.inner(),
                                //             ),
                                //         ),
                                //     ),
                                // ))
                                None
                            }
                        })
                        .into_iter()
                        // .chain([iced_runtime::core::Event::PlatformSpecific(
                        //     PlatformSpecific::Wayland(wayland::Event::Seat(
                        //         wayland::SeatEvent::Enter,
                        //         seat_id,
                        //     )),
                        // )])
                        .collect(),
                    KeyboardEventVariant::Press(ke) => {
                        let (key, location) = keysym_to_vkey_location(ke.keysym);
                        Some(iced_runtime::core::Event::Keyboard(
                            keyboard::Event::KeyPressed {
                                key: key,
                                location: location,
                                text: ke.utf8.map(|s| s.into()),
                                modifiers: modifiers_to_native(*modifiers),
                            },
                        ))
                        .into_iter()
                        .collect()
                    }
                    KeyboardEventVariant::Repeat(KeyEvent { keysym, utf8, .. }) => {
                        let (key, location) = keysym_to_vkey_location(keysym);
                        Some(iced_runtime::core::Event::Keyboard(
                            keyboard::Event::KeyPressed {
                                key: key,
                                location: location,
                                text: utf8.map(|s| s.into()),
                                modifiers: modifiers_to_native(*modifiers),
                            },
                        ))
                        .into_iter()
                        .collect()
                    }
                    KeyboardEventVariant::Release(ke) => {
                        let (k, location) = keysym_to_vkey_location(ke.keysym);
                        Some(iced_runtime::core::Event::Keyboard(
                            keyboard::Event::KeyReleased {
                                key: k,
                                location: location,
                                modifiers: modifiers_to_native(*modifiers),
                            },
                        ))
                        .into_iter()
                        .collect()
                    }
                    KeyboardEventVariant::Modifiers(new_mods) => {
                        *modifiers = new_mods;
                        vec![iced_runtime::core::Event::Keyboard(
                            keyboard::Event::ModifiersChanged(modifiers_to_native(new_mods)),
                        )]
                    }
                }
            }
            SctkEvent::TouchEvent {
                variant,
                touch_id: _,
                seat_id: _,
                surface: _,
            } => {
                vec![iced_runtime::core::Event::Touch(variant)]
            }
            SctkEvent::WindowEvent {
                variant,
                id: surface,
            } => match variant {
                // TODO Ashley: platform specific events for window
                WindowEventVariant::Created(..) => Default::default(),
                WindowEventVariant::Close => destroyed_surface_ids
                    .get(&surface.id())
                    .map(|id| iced_runtime::core::Event::Window(id.inner(), window::Event::Closed))
                    .into_iter()
                    .collect(),
                WindowEventVariant::WmCapabilities(caps) => Default::default(),
                // surface_ids
                // .get(&surface.id())
                // .map(|id| id.inner())
                // .map(|id| {
                //     iced_runtime::core::Event::PlatformSpecific(
                //         PlatformSpecific::Wayland(wayland::Event::Window(
                //             wayland::WindowEvent::WmCapabilities(caps),
                //             surface,
                //             id,
                //         )),
                //     )
                // })
                // .into_iter()
                // .collect(),
                WindowEventVariant::ConfigureBounds { .. } => Default::default(),
                WindowEventVariant::Configure((new_width, new_height), configure, surface, _) => {
                    surface_ids
                        .get(&surface.id())
                        .map(|id| {
                            if configure.is_resizing() {
                                vec![iced_runtime::core::Event::Window(
                                    id.inner(),
                                    window::Event::Resized {
                                        width: new_width.get(),
                                        height: new_height.get(),
                                    },
                                )]
                            } else {
                                vec![
                                    iced_runtime::core::Event::Window(
                                        id.inner(),
                                        window::Event::Resized {
                                            width: new_width.get(),
                                            height: new_height.get(),
                                        },
                                    ),
                                    // iced_runtime::core::Event::PlatformSpecific(
                                    //     PlatformSpecific::Wayland(wayland::Event::Window(
                                    //         wayland::WindowEvent::Configure(configure),
                                    //         surface,
                                    //         id.inner(),
                                    //     )),
                                    // ),
                                ]
                            }
                        })
                        .unwrap_or_default()
                }
                WindowEventVariant::ScaleFactorChanged(..) => Default::default(),
                WindowEventVariant::StateChanged(s) => Default::default(),
                // surface_ids
                // .get(&surface.id())
                // .map(|id| {
                //     iced_runtime::core::Event::PlatformSpecific(
                //         PlatformSpecific::Wayland(wayland::Event::Window(
                //             wayland::WindowEvent::State(s),
                //             surface,
                //             id.inner(),
                //         )),
                //     )
                // })
                // .into_iter()
                // .collect(),
                WindowEventVariant::Size(_, _, _) => vec![],
            },
            SctkEvent::LayerSurfaceEvent {
                variant,
                id: surface,
            } => match variant {
                LayerSurfaceEventVariant::Done => Default::default(),
                // destroyed_surface_ids
                // .get(&surface.id())
                // .map(|id| {
                //     iced_runtime::core::Event::PlatformSpecific(
                //         PlatformSpecific::Wayland(wayland::Event::Layer(
                //             LayerEvent::Done,
                //             surface,
                //             id.inner(),
                //         )),
                //     )
                // })
                // .into_iter()
                // .collect(),
                _ => Default::default(),
            },
            SctkEvent::PopupEvent {
                variant,
                id: surface,
                ..
            } => {
                match variant {
                    PopupEventVariant::Done => Default::default(),
                    // destroyed_surface_ids
                    // .get(&surface.id())
                    // .map(|id| {
                    //     iced_runtime::core::Event::PlatformSpecific(
                    //         PlatformSpecific::Wayland(
                    //             wayland::Event::Popup(
                    //                 PopupEvent::Done,
                    //                 surface,
                    //                 id.inner(),
                    //             ),
                    //         ),
                    //     )
                    // })
                    // .into_iter()
                    // .collect(),
                    PopupEventVariant::Created(_, _) => Default::default(), // TODO
                    PopupEventVariant::Configure(_, _, _) => Default::default(), // TODO
                    PopupEventVariant::RepositionionedPopup { token: _ } => Default::default(),
                    PopupEventVariant::Size(_, _) => Default::default(),
                    PopupEventVariant::ScaleFactorChanged(..) => Default::default(), // TODO
                }
            }
            SctkEvent::NewOutput { id, info } => {
                // Some(iced_runtime::core::Event::PlatformSpecific(
                //     PlatformSpecific::Wayland(wayland::Event::Output(
                //         wayland::OutputEvent::Created(info),
                //         id,
                //     )),
                // ))
                // .into_iter()
                // .collect()
                Default::default()
            }
            SctkEvent::UpdateOutput { id, info } => {
                // vec![iced_runtime::core::Event::PlatformSpecific(
                //     PlatformSpecific::Wayland(wayland::Event::Output(
                //         wayland::OutputEvent::InfoUpdate(info),
                //         id,
                //     )),
                // )]
                Default::default()
            }
            SctkEvent::RemovedOutput(id) => {
                // Some(iced_runtime::core::Event::PlatformSpecific(
                //     PlatformSpecific::Wayland(wayland::Event::Output(
                //         wayland::OutputEvent::Removed,
                //         id,
                //     )),
                // ))
                // .into_iter()
                // .collect()
                Default::default()
            }
            SctkEvent::ScaleFactorChanged {
                factor: _,
                id: _,
                inner_size: _,
            } => Default::default(),
            SctkEvent::DndOffer { event, surface } => match event {
                DndOfferEvent::Enter { mime_types, x, y } => {
                    // Some(iced_runtime::core::Event::PlatformSpecific(
                    //     PlatformSpecific::Wayland(wayland::Event::DndOffer(
                    //         wayland::DndOfferEvent::Enter { mime_types, x, y },
                    //     )),
                    // ))
                    // .into_iter()
                    // .collect()
                    Default::default()
                }
                DndOfferEvent::Motion { x, y } => {
                    let offset =
                        if let Some((x_offset, y_offset, _)) = subsurface_ids.get(&surface.id()) {
                            (*x_offset, *y_offset)
                        } else {
                            (0, 0)
                        };
                    // Some(iced_runtime::core::Event::PlatformSpecific(
                    //     PlatformSpecific::Wayland(wayland::Event::DndOffer(
                    //         wayland::DndOfferEvent::Motion {
                    //             x: x + offset.0 as f64,
                    //             y: y + offset.1 as f64,
                    //         },
                    //     )),
                    // ))
                    // .into_iter()
                    // .collect()
                    Default::default()
                }
                DndOfferEvent::DropPerformed => {
                    // Some(iced_runtime::core::Event::PlatformSpecific(
                    //     PlatformSpecific::Wayland(wayland::Event::DndOffer(
                    //         wayland::DndOfferEvent::DropPerformed,
                    //     )),
                    // ))
                    // .into_iter()
                    // .collect()
                    Default::default()
                }
                DndOfferEvent::Leave => {
                    // Some(iced_runtime::core::Event::PlatformSpecific(
                    //     PlatformSpecific::Wayland(wayland::Event::DndOffer(
                    //         wayland::DndOfferEvent::Leave,
                    //     )),
                    // ))
                    // .into_iter()
                    // .collect()
                    Default::default()
                }
                DndOfferEvent::Data { mime_type, data } => {
                    // Some(iced_runtime::core::Event::PlatformSpecific(
                    //     PlatformSpecific::Wayland(wayland::Event::DndOffer(
                    //         wayland::DndOfferEvent::DndData { data, mime_type },
                    //     )),
                    // ))
                    // .into_iter()
                    // .collect()
                    Default::default()
                }
                DndOfferEvent::SourceActions(actions) => {
                    // Some(iced_runtime::core::Event::PlatformSpecific(
                    //     PlatformSpecific::Wayland(wayland::Event::DndOffer(
                    //         wayland::DndOfferEvent::SourceActions(actions),
                    //     )),
                    // ))
                    // .into_iter()
                    // .collect()
                    Default::default()
                }
                DndOfferEvent::SelectedAction(action) => {
                    // Some(iced_runtime::core::Event::PlatformSpecific(
                    //     PlatformSpecific::Wayland(wayland::Event::DndOffer(
                    //         wayland::DndOfferEvent::SelectedAction(action),
                    //     )),
                    // ))
                    // .into_iter()
                    // .collect()
                    Default::default()
                }
            },
            SctkEvent::DataSource(event) => match event {
                DataSourceEvent::DndDropPerformed => {
                    // Some(iced_runtime::core::Event::PlatformSpecific(
                    //     PlatformSpecific::Wayland(wayland::Event::DataSource(
                    //         wayland::DataSourceEvent::DndDropPerformed,
                    //     )),
                    // ))
                    // .into_iter()
                    // .collect()
                    Default::default()
                }
                DataSourceEvent::DndFinished => {
                    // Some(iced_runtime::core::Event::PlatformSpecific(
                    //     PlatformSpecific::Wayland(wayland::Event::DataSource(
                    //         wayland::DataSourceEvent::DndFinished,
                    //     )),
                    // ))
                    // .into_iter()
                    // .collect()
                    Default::default()
                }
                DataSourceEvent::DndCancelled => {
                    // Some(iced_runtime::core::Event::PlatformSpecific(
                    //     PlatformSpecific::Wayland(wayland::Event::DataSource(
                    //         wayland::DataSourceEvent::Cancelled,
                    //     )),
                    // ))
                    // .into_iter()
                    // .collect()
                    Default::default()
                }
                DataSourceEvent::MimeAccepted(mime_type) => {
                    // Some(iced_runtime::core::Event::PlatformSpecific(
                    //     PlatformSpecific::Wayland(wayland::Event::DataSource(
                    //         wayland::DataSourceEvent::MimeAccepted(mime_type),
                    //     )),
                    // ))
                    // .into_iter()
                    // .collect()
                    Default::default()
                }
                DataSourceEvent::DndActionAccepted(action) => {
                    // Some(iced_runtime::core::Event::PlatformSpecific(
                    //     PlatformSpecific::Wayland(wayland::Event::DataSource(
                    //         wayland::DataSourceEvent::DndActionAccepted(action),
                    //     )),
                    // ))
                    // .into_iter()
                    // .collect()
                    Default::default()
                }
                DataSourceEvent::SendDndData { mime_type } => {
                    // Some(iced_runtime::core::Event::PlatformSpecific(
                    //     PlatformSpecific::Wayland(wayland::Event::DataSource(
                    //         wayland::DataSourceEvent::SendDndData(mime_type),
                    //     )),
                    // ))
                    // .into_iter()
                    // .collect()
                    Default::default()
                }
                DataSourceEvent::SendSelectionData { mime_type } => {
                    // Some(iced_runtime::core::Event::PlatformSpecific(
                    //     PlatformSpecific::Wayland(wayland::Event::DataSource(
                    //         wayland::DataSourceEvent::SendSelectionData(
                    //             mime_type,
                    //         ),
                    //     )),
                    // ))
                    // .into_iter()
                    // .collect()
                    Default::default()
                }
            },
            SctkEvent::SessionLocked => {
                // Some(iced_runtime::core::Event::PlatformSpecific(
                //     PlatformSpecific::Wayland(wayland::Event::SessionLock(
                //         wayland::SessionLockEvent::Locked,
                //     )),
                // ))
                // .into_iter()
                // .collect()
                Default::default()
            }
            SctkEvent::SessionLockFinished => {
                // Some(iced_runtime::core::Event::PlatformSpecific(
                //     PlatformSpecific::Wayland(wayland::Event::SessionLock(
                //         wayland::SessionLockEvent::Finished,
                //     )),
                // ))
                // .into_iter()
                // .collect()
                Default::default()
            }
            SctkEvent::SessionLockSurfaceCreated { .. } => vec![],
            SctkEvent::SessionLockSurfaceConfigure { .. } => vec![],
            SctkEvent::SessionLockSurfaceDone { .. } => vec![],
            SctkEvent::SessionUnlocked => {
                // Some(iced_runtime::core::Event::PlatformSpecific(
                //     PlatformSpecific::Wayland(wayland::Event::SessionLock(
                //         wayland::SessionLockEvent::Unlocked,
                //     )),
                // ))
                // .into_iter()
                // .collect()
                Default::default()
            }
            SctkEvent::SessionLockSurfaceScaleFactorChanged { .. } => vec![],
        }
    }
}

fn keysym_to_vkey_location(keysym: Keysym) -> (Key, Location) {
    let raw = keysym.raw();
    let mut key = keysym_to_key(raw);
    if matches!(key, key::Key::Unidentified) {
        // XXX is there a better way to do this?
        // we need to be able to determine the actual character for the key
        // not the combination, so this seems to be correct
        let mut utf8 = xkbcommon::xkb::keysym_to_utf8(keysym);
        // remove null terminator
        utf8.pop();
        if utf8.len() > 0 {
            key = Key::Character(utf8.into());
        }
    }

    let location = keymap::keysym_location(raw);
    (key, location)
}
