tonic::include_proto!("clipboard");

impl From<ClipboardType> for crate::ClipboardType {
    fn from(t: ClipboardType) -> crate::ClipboardType {
        match t {
            ClipboardType::Clipboard => crate::ClipboardType::Clipboard,
            ClipboardType::Primary => crate::ClipboardType::Primary,
        }
    }
}

impl From<crate::ClipboardType> for ClipboardType {
    fn from(t: crate::ClipboardType) -> ClipboardType {
        match t {
            crate::ClipboardType::Clipboard => ClipboardType::Clipboard,
            crate::ClipboardType::Primary => ClipboardType::Primary,
        }
    }
}

impl From<crate::ClipboardType> for i32 {
    fn from(t: crate::ClipboardType) -> i32 {
        t as i32
    }
}

impl From<crate::ClipboardData> for ClipboardData {
    fn from(data: crate::ClipboardData) -> ClipboardData {
        ClipboardData {
            id: data.id as u64,
            data: data.data,
            clipboard_type: data.clipboard_type.into(),
            timestamp: data
                .timestamp
                .duration_since(std::time::UNIX_EPOCH)
                .expect("duration since")
                .as_millis() as u64,
        }
    }
}
