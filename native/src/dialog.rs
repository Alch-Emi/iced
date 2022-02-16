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
use std::borrow::Borrow;
use std::borrow::Cow;
use std::fmt;
use std::path::PathBuf;

/// An action which triggers a dialog to open, and resolves when it closes
pub enum Action<Msg> {
    /// Produce a message dialog
    ///
    /// These dialogs normally take the form of a small window with a message
    /// and one or two buttons.
    MessageDialog(
        MessageDialogOptions,
        MessageDialogVariant<Msg>,
        /// The message that the message dialog should display
        Option<Cow<'static, str>>,
    ),

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
pub struct MessageDialogOptions {
    /// The severity to associate with this dialog
    pub level: MessageLevel,

    /// The dialog window's title
    pub title: Option<Cow<'static, str>>,
}

impl MessageDialogOptions {
    /// Create a new message dialog with a severity of INFO
    pub const fn new() -> Self {
        Self {
            level: MessageLevel::Info,
            title: None,
        }
    }

    /// Create a new message dialog with a severity of INFO and the given title
    pub const fn new_with_title(title: &'static str) -> Self {
        Self {
            level: MessageLevel::Info,
            title: Some(Cow::Borrowed(title)),
        }
    }

    /// Set the title of the message dialog
    pub fn with_title(self, title: impl ToString) -> Self {
        Self {
            title: Some(Cow::Owned(title.to_string())),
            ..self
        }
    }

    /// Set the severity of the message dialog
    pub fn with_level(self, level: MessageLevel) -> Self {
        Self { level, ..self }
    }
}

/// Possible severity levels for message dialogs
#[derive(Debug)]
pub enum MessageLevel {
    /// Give the user some information
    ///
    /// This typically doesn't require any urgent action, and does not bode
    /// unwell.
    Info,

    /// Alert the user of a non-fatal problem
    ///
    /// Execution can continue, but may imply a risk, or could lead to an error
    /// or unexpected behaviour down the line
    Warning,

    /// There was a serious problem, and the program couldn't complete its goal
    Error,
}

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
pub struct FileDialogOptions {
    /// A list of filters that should be available to the user
    ///
    /// Defaults to no filters (e.g. show all files)
    pub filters: Cow<'static, [Filter]>,

    /// Sets the directory that the prompt will start on
    pub initial_directory: Option<PathBuf>,

    /// Sets the initial value of the filename
    ///
    /// Particularly useful for save dialogs
    pub initial_filename: Option<Cow<'static, str>>,

    /// Sets the dialog window title
    pub title: Option<Cow<'static, str>>,
}

impl FileDialogOptions {
    /// Create a new [`FileDialogOptions`] with the default values
    pub const fn new() -> Self {
        Self::new_with_filters(&[])
    }

    /// Create a new [`FileDialogOptions`] with a set of filters
    ///
    /// All other values are set to defaults
    pub const fn new_with_filters(filters: &'static [Filter]) -> Self {
        FileDialogOptions {
            filters: Cow::Borrowed(filters),
            initial_directory: None,
            initial_filename: None,
            title: None,
        }
    }

    /// Add a new filter to the existing filters
    pub fn add_filter(mut self, filter: Filter) -> Self {
        self.filters.to_mut().push(filter);
        self
    }

    /// Set the initial directory of the dialog
    pub fn with_initial_directory(self, path: PathBuf) -> Self {
        Self {
            initial_directory: Some(path),
            ..self
        }
    }

    /// Set the initial filename of the dialog
    pub fn with_initial_filename(self, filename: impl ToString) -> Self {
        Self {
            initial_filename: Some(Cow::Owned(filename.to_string())),
            ..self
        }
    }

    /// Set the dialog's title
    pub fn with_title(self, title: impl ToString) -> Self {
        Self {
            title: Some(Cow::Owned(title.to_string())),
            ..self
        }
    }
}

/// A filter for the kinds of files that can be selected from a file dialog
///
/// This often takes the form of a category of file (say, "Music Files") along
/// with a series of file extensions that fall into that category ("flac",
/// "opus", "mp3")
///
/// The user will be able to use these filters to make finding the file they are
/// looking for easier.  However, these filters don't restrict the user, so
/// there's no guarantee that the selected file will comply with one of the
/// filters.
#[derive(Debug, Clone)]
pub enum Filter {
    /// The normal form of a filter, with owned values for names and extensions
    Owned {
        /// The name of the filter (e.g. "Music Files")
        name: String,
        /// The list of valid extensions (e.g. "mp3", ...)
        extensions: Vec<String>,
    },

    /// A compile-time constant filter
    ///
    /// This is practically identical to the [`Self::Owned`] variant, but can
    /// (and must) be a compile time constant
    Static {
        /// The name of the filter (e.g. "Music Files")
        name: &'static str,
        /// The list of valid extensions (e.g. "mp3", ...)
        extensions: &'static [&'static str],
    },
}

impl Filter {
    /// Construct a new [`Filter`] using `&'static str`s
    ///
    /// ## Example
    /// ```
    /// # use iced_native::dialog::Filter;
    /// let name = "Music Files";
    /// let extensions = &["mp3", "flac", "ogg"];
    /// let filter = Filter::new_const(name, extensions);
    /// # assert_eq!(filter.name(), "Music Files");
    /// # assert_eq!(
    /// #   filter.extensions(),
    /// #   vec![
    /// #       "mp3".to_owned(),
    /// #       "flac".to_owned(),
    /// #       "ogg".to_owned()
    /// #   ]
    /// # );
    /// ```
    pub const fn new_const(
        name: &'static str,
        extensions: &'static [&'static str],
    ) -> Self {
        Self::Static { name, extensions }
    }

    /// Construct a new [`Filter`]
    ///
    /// ## Example
    /// ```
    /// # use iced_native::dialog::Filter;
    /// let name = "Music Files".to_string();
    /// let extensions = vec!["mp3".to_string(), "flac".to_string(), "ogg".to_string()];
    /// let filter = Filter::new(name, extensions);
    /// # assert_eq!(filter.name(), "Music Files");
    /// # assert_eq!(
    /// #   filter.extensions(),
    /// #   vec![
    /// #       "mp3".to_owned(),
    /// #       "flac".to_owned(),
    /// #       "ogg".to_owned()
    /// #   ]
    /// # );
    /// ```
    pub fn new(name: impl Into<String>, extensions: Vec<String>) -> Self {
        Self::Owned {
            name: name.into(),
            extensions,
        }
    }

    /// Get the filter name
    pub fn name(&self) -> &str {
        match self {
            Self::Static { name, .. } => name,
            Self::Owned { name, .. } => name.as_str(),
        }
    }

    /// Get a list of extensions in this filter
    pub fn extensions(&self) -> Vec<String> {
        match self {
            Self::Static { extensions, .. } => {
                extensions.iter().map(|s| (*s).to_owned()).collect()
            }
            Self::Owned { extensions, .. } => extensions.clone(),
        }
    }
}

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
            Self::MessageDialog(options, variant, text) => {
                Action::MessageDialog(options, variant.map(f), text)
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
            Action::MessageDialog(options, variant, text) => {
                write!(
                    f,
                    "MessageDialog({:?}, {:?} {:?})",
                    options,
                    variant,
                    text.borrow()
                )
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
