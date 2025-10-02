//! An [`Application`] wrapper around an [`IcedEditor`] to bridge between `iced_baseview` and
//! `nih_plug_iced`.

use crossbeam::channel;
use iced_baseview::{
    baseview::WindowScalePolicy, core::Element, futures::{Subscription, subscription::{EventStream, Hasher, Recipe, from_recipe}}, window::WindowSubs,
    Renderer, Task,
};
use futures_util::stream::BoxStream;
use nih_plug::prelude::GuiContext;
use std::sync::Arc;
use std::hash::Hash;

use crate::{IcedEditor, ParameterUpdate};

/// A custom subscription recipe for parameter updates from a crossbeam channel
struct ParameterUpdatesRecipe {
    receiver: Arc<channel::Receiver<ParameterUpdate>>,
}

impl Recipe for ParameterUpdatesRecipe {
    type Output = ();

    fn hash(&self, state: &mut Hasher) {
        // Use a constant ID since we only have one parameter updates subscription
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(self: Box<Self>, _input: EventStream) -> BoxStream<'static, Self::Output> {
        Box::pin(futures_util::stream::unfold(
            self.receiver,
            |receiver| async move {
                match receiver.try_recv() {
                    Ok(_) => Some(((), receiver)),
                    Err(channel::TryRecvError::Empty) => {
                        // Wait a bit before checking again to avoid busy-waiting
                        futures_util::future::pending::<()>().await;
                        None
                    }
                    Err(channel::TryRecvError::Disconnected) => None,
                }
            },
        ))
    }
}

/// Wraps an `iced_baseview` [`Application`] around [`IcedEditor`]. Needed to allow editors to
/// always receive a copy of the GUI context.
pub(crate) struct IcedEditorWrapperApplication<E: IcedEditor> {
    editor: E,

    /// We will receive notifications about parameters being changed on here. Whenever a parameter
    /// update gets sent, we will trigger a [`Message::parameterUpdate`] which causes the UI to be
    /// redrawn.
    parameter_updates_receiver: Arc<channel::Receiver<ParameterUpdate>>,
}

/// This wraps around `E::Message` to add a parameter update message which can be handled directly
/// by this wrapper. That parameter update message simply forces a redraw of the GUI whenever there
/// is a parameter update.
pub enum Message<E: IcedEditor> {
    EditorMessage(E::Message),
    ParameterUpdate,
}

impl<E: IcedEditor> Message<E> {
    fn into_editor_message(self) -> Option<E::Message> {
        if let Message::EditorMessage(message) = self {
            Some(message)
        } else {
            None
        }
    }
}

impl<E: IcedEditor> std::fmt::Debug for Message<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EditorMessage(arg0) => f.debug_tuple("EditorMessage").field(arg0).finish(),
            Self::ParameterUpdate => write!(f, "ParameterUpdate"),
        }
    }
}

impl<E: IcedEditor> Clone for Message<E> {
    fn clone(&self) -> Self {
        match self {
            Self::EditorMessage(arg0) => Self::EditorMessage(arg0.clone()),
            Self::ParameterUpdate => Self::ParameterUpdate,
        }
    }
}

impl<E: IcedEditor> iced_baseview::Application for IcedEditorWrapperApplication<E> {
    type Executor = E::Executor;
    type Message = Message<E>;
    type Flags = (
        Arc<dyn GuiContext>,
        Arc<channel::Receiver<ParameterUpdate>>,
        E::InitializationFlags,
    );
    type Theme = E::Theme;

    fn new(
        (context, parameter_updates_receiver, flags): Self::Flags,
    ) -> (Self, Task<Self::Message>) {
        let (editor, task) = E::new(flags, context);

        (
            Self {
                editor,
                parameter_updates_receiver,
            },
            task.map(Message::EditorMessage),
        )
    }

    #[inline]
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::EditorMessage(message) => {
                self.editor.update(message).map(Message::EditorMessage)
            }
            Message::ParameterUpdate => Task::none(),
        }
    }

    #[inline]
    fn subscription(
        &self,
        window_subs: &mut WindowSubs<Self::Message>,
    ) -> Subscription<Self::Message> {
        // Since we're wrapping around `E::Message`, we need to do this transformation ourselves
        let on_frame = window_subs.on_frame.clone();
        let on_window_will_close = window_subs.on_window_will_close.clone();
        let on_resize = window_subs.on_resize.clone();
        let mut editor_window_subs: WindowSubs<E::Message> = WindowSubs {
            on_frame: Some(Arc::new(move || {
                let cb = on_frame.clone();
                cb.and_then(|cb| cb().and_then(|m| m.into_editor_message()))
            })),
            on_window_will_close: Some(Arc::new(move || {
                let cb = on_window_will_close.clone();
                cb.and_then(|cb| cb().and_then(|m| m.into_editor_message()))
            })),
            on_resize: on_resize.clone().map(|cb| {
                Arc::new(move |size| {
                    cb(size).and_then(|m| m.into_editor_message())
                }) as Arc<dyn Fn(iced_baseview::Size) -> Option<E::Message>>
            }),
        };

        let subscription = Subscription::batch([
            from_recipe(ParameterUpdatesRecipe {
                receiver: self.parameter_updates_receiver.clone(),
            })
            .map(|_| Message::ParameterUpdate),
            self.editor
                .subscription(&mut editor_window_subs)
                .map(Message::EditorMessage),
        ]);

        if let Some(message) = editor_window_subs.on_frame.as_ref() {
            let message = Arc::clone(message);
            window_subs.on_frame = Some(Arc::new(move || message().map(Message::EditorMessage)));
        }
        if let Some(message) = editor_window_subs.on_window_will_close.as_ref() {
            let message = Arc::clone(message);
            window_subs.on_window_will_close =
                Some(Arc::new(move || message().map(Message::EditorMessage)));
        }

        subscription
    }

    #[inline]
    fn view(&self) -> Element<'_, Self::Message, Self::Theme, Renderer> {
        self.editor.view().map(Message::EditorMessage)
    }

    #[inline]
    fn scale_policy(&self) -> WindowScalePolicy {
        WindowScalePolicy::SystemScaleFactor
    }

    fn title(&self) -> String {
        self.editor.title()
    }

    #[inline]
    fn theme(&self) -> Self::Theme {
        self.editor.theme()
    }
}
