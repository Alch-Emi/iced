use crate::clipboard;
use crate::dialog;
use crate::window;

use iced_futures::MaybeSend;

use std::fmt;

/// An action that a [`Command`] can perform.
///
/// [`Command`]: crate::Command
pub enum Action<T> {
    /// Run a [`Future`] to completion.
    Future(iced_futures::BoxFuture<T>),

    /// Run a clipboard action.
    Clipboard(clipboard::Action<T>),

    /// Run a window action.
    Window(window::Action),

    /// Open a file dialog
    Dialog(dialog::Action<T>),
}

impl<T> Action<T> {
    /// Applies a transformation to the result of a [`Command`].
    pub fn map<A>(
        self,
        f: impl Fn(T) -> A + 'static + MaybeSend + Sync,
    ) -> Action<A>
    where
        T: 'static,
    {
        use iced_futures::futures::FutureExt;

        match self {
            Self::Future(future) => Action::Future(Box::pin(future.map(f))),
            Self::Clipboard(action) => Action::Clipboard(action.map(f)),
            Self::Window(window) => Action::Window(window),
            Self::Dialog(dialog) => Action::Dialog(dialog.map(f)),
        }
    }
}

impl<T> fmt::Debug for Action<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Future(_) => write!(f, "Action::Future"),
            Self::Clipboard(action) => {
                write!(f, "Action::Clipboard({:?})", action)
            }
            Self::Window(action) => write!(f, "Action::Window({:?})", action),
            Self::Dialog(action) => write!(f, "Action::Dialog({:?})", action),
        }
    }
}
