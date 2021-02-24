use clipcat::ClipEntry;

pub const ENTRY_SEPARATOR: &str = "\n";
pub const INDEX_SEPARATOR: char = ':';

pub trait FinderStream: Send + Sync {
    fn generate_input(&self, clips: &[ClipEntry]) -> String {
        clips
            .iter()
            .enumerate()
            .map(|(i, data)| {
                format!("{}{} {}", i, INDEX_SEPARATOR, data.printable_data(self.line_length()))
            })
            .collect::<Vec<_>>()
            .join(ENTRY_SEPARATOR)
    }

    fn parse_output(&self, data: &[u8]) -> Vec<usize> {
        let line = String::from_utf8_lossy(data);
        line.split(ENTRY_SEPARATOR)
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

    fn set_line_length(&mut self, _line_length: usize) {}

    fn set_menu_length(&mut self, _menu_length: usize) {}

    fn menu_length(&self) -> Option<usize> { None }

    fn line_length(&self) -> Option<usize> { None }
}

#[cfg(test)]
mod tests {
    use clipcat::{ClipEntry, ClipboardMode};

    use crate::finder::FinderStream;

    struct Dummy;
    impl FinderStream for Dummy {}

    #[test]
    fn test_generate_input() {
        const MODE: ClipboardMode = ClipboardMode::Clipboard;
        let d = Dummy;
        let clips = vec![];
        let v = d.generate_input(&clips);
        assert_eq!(v, "");

        let clips = vec![ClipEntry::from_string("abcde", MODE)];
        let v = d.generate_input(&clips);
        assert_eq!(v, "0: abcde");

        let clips = vec![
            ClipEntry::from_string("abcde", MODE),
            ClipEntry::from_string("АбВГД", MODE),
            ClipEntry::from_string("あいうえお", MODE),
        ];

        let v = d.generate_input(&clips);
        assert_eq!(v, "0: abcde\n1: АбВГД\n2: あいうえお");
    }

    #[test]
    fn test_parse_output() {
        let d = Dummy;
        let output = "";
        let v = d.parse_output(&output.as_bytes());
        assert!(v.is_empty());

        let output = ":";
        let v = d.parse_output(&output.as_bytes());
        assert!(v.is_empty());

        let output = "::::::::";
        let v = d.parse_output(&output.as_bytes());
        assert!(v.is_empty());

        let output = "\n\n\n\n\n";
        let v = d.parse_output(&output.as_bytes());
        assert!(v.is_empty());

        let output = "9\n3\n0\n4\n1\n";
        let v = d.parse_output(&output.as_bytes());
        assert_eq!(v, &[9, 3, 0, 4, 1]);

        let output = "203: abcde|АбВГД3|200あいうえお385";
        let v = d.parse_output(&output.as_bytes());
        assert_eq!(v, &[203]);

        let output = "2:3:4:5";
        let v = d.parse_output(&output.as_bytes());
        assert_eq!(v, &[2]);

        let output = "10: abcde\n2: АбВГД3020\n9:333\n7:30あいうえお38405\n1:323";
        let v = d.parse_output(&output.as_bytes());
        assert_eq!(v, &[10, 2, 9, 7, 1]);
    }
}
