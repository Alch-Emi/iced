//! Open all sorts of dialogs
//!
//! The `dialog` module provides a convenient way to open dialogs, including
//! file dialogs, yes/no and informational dialogs.
//!
//! The centerpiece of this module is the [`Action`] enum, which can be used in
//! a [`Command`] in order to actually open the dialog.  If you're looking for a
//! place to start, try there at first.
//!
//! Under the hood, these are just convenient wrappers around [`rfd` (Rusty File
//! Dialog)](https://crates.io/crates/rfd)
//!
//! [`Command`]: iced_native::Command

use iced_futures::MaybeSend;
use std::fmt;
use std::path::PathBuf;

/// An action which triggers a dialog to open, and resolves when it closes
pub enum Action<Msg> {
    /// Produce a message dialog
    ///
    /// These dialogs normally take the form of a small window with a message
    /// and one or two buttons.
    MessageDialog(MessageDialogOptions, MessageDialogVariant<Msg>),

    /// Produce a file dialog
    ///
    /// File dialogs typically take the form of a paired-down file browser,
    /// which can be used to select one (or several) paths.
    ///
    /// Different restrictions can restrict what kinds of things the user can
    /// select, and how many of them.
    FileDialog(FileDialogOptions, FileDialogVariant<Msg>),
}

/// Various options common to all message dialogs
#[derive(Debug)]
pub struct MessageDialogOptions {}

/// Different variants of message dialogs
///
/// Most platforms support multiple different variants of message dialogs, some
/// of which have additional options, and each of which has different ways of
/// producing messages.
pub enum MessageDialogVariant<Msg> {
    /// Open a confirmation message dialog
    ///
    /// These are yes/no or confirm/cancel dialog boxes.  They normally take the
    /// form of a small pop-up with a title, brief message, and two buttons.
    Confirmation {
        /// The message that will be produced when the user closes the dialog
        ///
        /// If the user selected Yes/Okay, then `true` will be passed to the
        /// function.  Otherwise, if the user select No/Cancel or closed the
        /// dialog without selecting an option, `false` will be passed instead.
        on_close: Box<dyn FnOnce(bool) -> Msg>,

        /// If `true`, use Yes/No buttons, otherwise, use Okay/Cancel buttons
        is_yes_no: bool,
    },

    /// Open an informational message dialog
    ///
    /// Like the confirmation message dialog, this is a small pop-up with a
    /// title and a brief message, but the only option is an "Okay" button.
    Informational(
        /// The message produced when the user closes the dialog
        Msg,
    ),
}

/// Assorted options and filters that can be applied to any kind of file dialog
#[derive(Debug)]
pub struct FileDialogOptions {}

/// Different variants on file dialogs
///
/// File dialogs come in many different forms, and can be applied to a broad
/// range of uses.  This enum enumerates some of the common broad kinds of file
/// dialogs, and the options that accompany each.  Different kinds of dialogs
/// may produce messages in different ways
pub enum FileDialogVariant<Msg> {
    /// Open a single file dialog
    ///
    /// This selects a single file path, for either saving or opening
    #[doc(alias = "SaveDialog")]
    SingleFileDialog {
        /// Whether this is a save dialog, as opposed to an open dialog
        ///
        /// When set to `true`, the user will be able to select a non-existant
        /// path (although it must be in a valid directory).  This is useful for
        /// selecting where to save files.
        ///
        /// When `false`, only existing files will be available
        is_save_dialog: bool,

        /// The message that will be produced when the dialog is closed.
        ///
        /// If the user selected a file, then [`Some`] will be passed, along
        /// with the path of the selected file.  If the user closed the dialog
        /// without selecting a file, then [`None`] will be passed instead.
        on_select: Box<dyn FnOnce(Option<PathBuf>) -> Msg>,
    },

    /// Open a file dialog that can select more than one file
    ///
    /// This allows a user to select as many files as they please, but it cannot
    /// be used for a save operation.
    MultiFileDialog(
        /// The message that will be produced when the dialog is closed.
        ///
        /// If the user selected one or more files, then the function will be
        /// passed a [`Vec`] of file paths.  If the user closed the dialog
        /// without selecting any files, (for example, by cancelling), then the
        /// [`Vec`] will be empty.
        Box<dyn FnOnce(Vec<PathBuf>) -> Msg>,
    ),

    /// Open a file dialog that can open an entire folder.
    ///
    /// Instead of selecting just one file, allow the user to select a
    /// directory.
    FolderSelectDialog(
        /// The message that will be produced when the dialog is closed.
        ///
        /// If the user selected a file, then [`Some`] will be passed, along
        /// with the path of the selected file.  If the user closed the dialog
        /// without selecting a file, then [`None`] will be passed instead.
        Box<dyn FnOnce(Option<PathBuf>) -> Msg>,
    ),
}

impl<Msg> Action<Msg> {
    /// Apply some sort of transformation to the message produced by this action
    pub fn map<MappedMsg, Mapper>(self, f: Mapper) -> Action<MappedMsg>
    where
        Msg: 'static,
        Mapper: FnOnce(Msg) -> MappedMsg + 'static + MaybeSend + Sync,
    {
        match self {
            Self::MessageDialog(options, variant) => {
                Action::MessageDialog(options, variant.map(f))
            }
            Self::FileDialog(options, variant) => {
                Action::FileDialog(options, variant.map(f))
            }
        }
    }
}

impl<Msg> MessageDialogVariant<Msg> {
    /// Apply some transformation to the message produced by this variant
    pub fn map<MappedMsg, Mapper>(
        self,
        f: Mapper,
    ) -> MessageDialogVariant<MappedMsg>
    where
        Msg: 'static,
        Mapper: FnOnce(Msg) -> MappedMsg + 'static + MaybeSend + Sync,
    {
        match self {
            Self::Confirmation {
                on_close,
                is_yes_no,
            } => MessageDialogVariant::Confirmation {
                on_close: Box::new(move |choice| f(on_close(choice))),
                is_yes_no,
            },
            Self::Informational(on_close) => {
                MessageDialogVariant::Informational(f(on_close))
            }
        }
    }
}

impl<Msg> FileDialogVariant<Msg> {
    /// Apply some transformation to the message produced by this variant
    pub fn map<MappedMsg, Mapper>(
        self,
        f: Mapper,
    ) -> FileDialogVariant<MappedMsg>
    where
        Msg: 'static,
        Mapper: FnOnce(Msg) -> MappedMsg + 'static + MaybeSend + Sync,
    {
        match self {
            Self::SingleFileDialog {
                is_save_dialog,
                on_select,
            } => FileDialogVariant::SingleFileDialog {
                on_select: Box::new(|file| f(on_select(file))),
                is_save_dialog,
            },
            Self::MultiFileDialog(on_select) => {
                FileDialogVariant::MultiFileDialog(Box::new(|files| {
                    f(on_select(files))
                }))
            }
            Self::FolderSelectDialog(on_select) => {
                FileDialogVariant::FolderSelectDialog(Box::new(|folder| {
                    f(on_select(folder))
                }))
            }
        }
    }
}

impl<T> fmt::Debug for Action<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::MessageDialog(options, variant) => {
                write!(f, "MessageDialog({:?}, {:?})", options, variant)
            }
            Action::FileDialog(options, variant) => {
                write!(f, "FileDialog({:?}, {:?})", options, variant)
            }
        }
    }
}

impl<T> fmt::Debug for MessageDialogVariant<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Confirmation {
                is_yes_no: true, ..
            } => write!(f, "Confirmation(Yes/No)"),
            Self::Confirmation {
                is_yes_no: false, ..
            } => write!(f, "Confirmation(Okay/Cancel)"),
            Self::Informational { .. } => write!(f, "Informational"),
        }
    }
}

impl<T> fmt::Debug for FileDialogVariant<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SingleFileDialog {
                is_save_dialog: true,
                ..
            } => write!(f, "SingleFileDialog(open)"),
            Self::SingleFileDialog {
                is_save_dialog: false,
                ..
            } => write!(f, "SingleFileDialog(save)"),
            Self::MultiFileDialog { .. } => write!(f, "MultiFileDialog"),
            Self::FolderSelectDialog { .. } => write!(f, "FolderSelectDialog"),
        }
    }
}
