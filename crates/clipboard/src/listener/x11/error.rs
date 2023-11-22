use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Xfixes is not present"))]
    XfixesNotPresent,

    #[snafu(display("Reply error: {source}"))]
    Reply { source: x11rb::errors::ReplyError, backtrace: snafu::Backtrace },

    #[snafu(display("Could not create X11 connection, error: {source}"))]
    Connect { source: x11rb::errors::ConnectError, backtrace: snafu::Backtrace },

    #[snafu(display("Could not generate X11 identifier, error: {source}"))]
    GenerateX11Identifier { source: x11rb::errors::ReplyOrIdError, backtrace: snafu::Backtrace },

    #[snafu(display("Could not create new window, error: {source}"))]
    CreateWindow { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[snafu(display("Could not flush connection, error: {source}"))]
    FlushConnection { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[snafu(display("Could not send event, error: {source}"))]
    SendEvent { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[snafu(display("Could not get selection owner, error: {source}"))]
    GetSelectionOwner { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[snafu(display("Could not claim selection owner, error: {source}"))]
    ClaimSelectionOwner { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[snafu(display("Error occurs while releasing selection owner, error: {source}"))]
    ReleaseSelectionOwner { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[snafu(display("Selection owner is not matched"))]
    MatchSelectionOwner,

    #[snafu(display("Could not change property, error: {source}"))]
    ChangeProperty { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[snafu(display("Could not change window attributes, error: {source}"))]
    ChangeWindowAttributes { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[snafu(display("Could not get atom identifier by name {atom_name}, error: {source}"))]
    GetAtomIdentifierByName {
        atom_name: String,
        source: x11rb::errors::ConnectionError,
        backtrace: snafu::Backtrace,
    },

    #[snafu(display("Could not get atom name, error: {source}"))]
    GetAtomName { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[snafu(display("Could not get property reply, error: {source}"))]
    GetPropertyReply { source: x11rb::errors::ReplyError, backtrace: snafu::Backtrace },

    #[snafu(display("Could not get property, error: {source}"))]
    GetProperty { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[snafu(display("Could not convert selection, error: {source}"))]
    ConvertSelection { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[snafu(display("Could not delete property, error: {source}"))]
    DeleteProperty { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[snafu(display("Error occurs while waiting for event, error: {source}"))]
    WaitForEvent { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[snafu(display("Error occurs while polling for event, error: {source}"))]
    PollForEvent { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[snafu(display("Could not query extension: {extension_name}, error: {source}"))]
    QueryExtension {
        extension_name: String,
        source: x11rb::errors::ConnectionError,
        backtrace: snafu::Backtrace,
    },

    #[snafu(display("Could not query Xfixes version, error: {source}"))]
    QueryXfixesVersion { source: x11rb::errors::ConnectionError, backtrace: snafu::Backtrace },

    #[snafu(display("Could not select Xfixes selection input, error: {source}"))]
    SelectXfixesSelectionInput {
        source: x11rb::errors::ConnectionError,
        backtrace: snafu::Backtrace,
    },
}
