use clipcat_base::ClipEntryMetadata;

pub const ENTRY_SEPARATOR: &str = "\n";
pub const INDEX_SEPARATOR: char = ':';

pub trait FinderStream: Send + Sync {
    fn generate_input(&self, clips: &[ClipEntryMetadata]) -> String {
        clips
            .iter()
            .enumerate()
            .map(|(i, ClipEntryMetadata { preview, .. })| format!("{i}{INDEX_SEPARATOR} {preview}"))
            .collect::<Vec<_>>()
            .join(ENTRY_SEPARATOR)
    }

    fn parse_output(&self, data: &[u8]) -> Vec<usize> {
        String::from_utf8_lossy(data)
            .split(ENTRY_SEPARATOR)
            .filter_map(|entry| {
                entry
                    .split(INDEX_SEPARATOR)
                    .next()
                    .expect("first part must exist")
                    .parse::<usize>()
                    .ok()
            })
            .collect()
    }

    fn set_extra_arguments(&mut self, _arguments: &[String]) {}

    fn set_line_length(&mut self, _line_length: usize) {}

    fn set_menu_length(&mut self, _menu_length: usize) {}

    fn menu_length(&self) -> Option<usize> { None }

    fn line_length(&self) -> Option<usize> { None }
}

#[cfg(test)]
mod tests {
    use clipcat_base::{ClipEntry, ClipboardKind};

    use crate::finder::FinderStream;

    struct Dummy;
    impl FinderStream for Dummy {}

    #[test]
    fn test_generate_input() {
        const KIND: ClipboardKind = ClipboardKind::Clipboard;
        let d = Dummy;
        let clips = vec![];
        let v = d.generate_input(&clips);
        assert_eq!(v, "");

        let clips = vec![ClipEntry::from_string("abcde", KIND).metadata(None)];
        let v = d.generate_input(&clips);
        assert_eq!(v, "0: abcde");

        let clips = vec![
            ClipEntry::from_string("abcde", KIND).metadata(None),
            ClipEntry::from_string("АбВГД", KIND).metadata(None),
            ClipEntry::from_string("あいうえお", KIND).metadata(None),
        ];

        let v = d.generate_input(&clips);
        assert_eq!(v, "0: abcde\n1: АбВГД\n2: あいうえお");
    }

    #[test]
    fn test_parse_output() {
        let d = Dummy;
        let output = "";
        let v = d.parse_output(output.as_bytes());
        assert!(v.is_empty());

        let output = ":";
        let v = d.parse_output(output.as_bytes());
        assert!(v.is_empty());

        let output = "::::::::";
        let v = d.parse_output(output.as_bytes());
        assert!(v.is_empty());

        let output = "\n\n\n\n\n";
        let v = d.parse_output(output.as_bytes());
        assert!(v.is_empty());

        let output = "9\n3\n0\n4\n1\n";
        let v = d.parse_output(output.as_bytes());
        assert_eq!(v, &[9, 3, 0, 4, 1]);

        let output = "203: abcde|АбВГД3|200あいうえお385";
        let v = d.parse_output(output.as_bytes());
        assert_eq!(v, &[203]);

        let output = "2:3:4:5";
        let v = d.parse_output(output.as_bytes());
        assert_eq!(v, &[2]);

        let output = "10: abcde\n2: АбВГД3020\n9:333\n7:30あいうえお38405\n1:323";
        let v = d.parse_output(output.as_bytes());
        assert_eq!(v, &[10, 2, 9, 7, 1]);
    }
}
