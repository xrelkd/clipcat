use std::fmt;

/// Checks if the error is related to gRPC message size limits
#[must_use]
pub fn is_grpc_message_size_error(error_str: &str) -> bool {
    error_str.contains("decoded message length too large")
}

/// Formats an error with additional help text if it's a message size error
pub fn format_error_with_help<E: fmt::Display>(
    error: &E,
    program_name: &str,
    config_file: &str,
) -> String {
    let error_str = error.to_string();

    if is_grpc_message_size_error(&error_str) {
        format!(
            "{error_str}\n\nThis error occurs when clipboard content exceeds the gRPC message \
             size limit.\nTo fix this, increase the 'grpc_max_message_size' value in your \
             {program_name} configuration file:\n\ngrpc_max_message_size = 16777216  # \
             16MB\n\nDefault location: $XDG_CONFIG_HOME/clipcat/{config_file}"
        )
    } else {
        error_str
    }
}
