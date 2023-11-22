use std::time::{Duration, Instant};

use clipcat::ClipboardContent;
use clipcat_clipboard::{
    ClipboardLoad, ClipboardLoadExt, ClipboardStore, ClipboardStoreExt, ClipboardSubscribe,
    ClipboardWait, Error,
};

pub trait ClipboardTester {
    type Clipboard: 'static
        + Clone
        + Sync
        + Send
        + ClipboardSubscribe
        + ClipboardLoad
        + ClipboardStore;

    fn new_clipboard(&self) -> Self::Clipboard;

    fn run(&self) -> Result<(), Error> {
        self.test_clean()?;

        for i in 0..20 {
            let data_size = 1 << i;
            println!("test with data size: {data_size}");
            self.test_store_and_load(data_size)?;
            println!("passed, test with data size: {data_size}");
        }

        self.test_subscribe()?;
        Ok(())
    }

    fn test_store_and_load(&self, len: usize) -> Result<(), Error> {
        let clipboard = self.new_clipboard();

        let original_data: String = vec!['A'; len].into_iter().collect();
        clipboard.store(ClipboardContent::Plaintext(original_data.clone()))?;

        for _ in 0..5 {
            std::thread::sleep(Duration::from_millis(20));
            if let ClipboardContent::Plaintext(loaded_data) = clipboard.load()? {
                assert_eq!(loaded_data.len(), original_data.len());
                assert_eq!(loaded_data, original_data);
            } else {
                panic!("Content type is not matched");
            }
        }

        Ok(())
    }

    fn test_clean(&self) -> Result<(), Error> {
        let data = "This is a string";
        let clipboard = self.new_clipboard();

        clipboard.store(ClipboardContent::Plaintext(data.to_string()))?;
        assert_eq!(clipboard.load().unwrap(), ClipboardContent::Plaintext(data.to_string()));

        clipboard.clear()?;
        assert!(match clipboard.load() {
            Ok(ClipboardContent::Plaintext(s)) => s.is_empty(),
            Err(Error::Empty) => true,
            _ => false,
        });

        Ok(())
    }

    fn test_subscribe(&self) -> Result<(), Error> {
        let clipboard = self.new_clipboard();
        clipboard.clear()?;

        let observer1 = std::thread::spawn({
            let subscriber = clipboard.subscribe()?;
            let clipboard = clipboard.clone();
            move || -> Result<String, Error> {
                loop {
                    let _unused = subscriber.wait();
                    match clipboard.load_string() {
                        Ok(data) => return Ok(data),
                        Err(Error::Empty) => continue,
                        Err(err) => return Err(err),
                    }
                }
            }
        });

        let observer2 = std::thread::spawn({
            let subscriber = clipboard.subscribe()?;
            let clipboard = clipboard.clone();
            move || -> Result<String, Error> {
                loop {
                    let _unused = subscriber.wait();
                    match clipboard.load_string() {
                        Ok(data) => return Ok(data),
                        Err(Error::Empty) => continue,
                        Err(err) => return Err(err),
                    }
                }
            }
        });

        let observer3 = std::thread::spawn({
            let subscriber = clipboard.subscribe()?;
            move || -> Result<(), Error> {
                while subscriber.wait().is_ok() {}
                Ok(())
            }
        });

        std::thread::sleep(Duration::from_millis(100));
        let input = format!("{:?}", Instant::now());
        clipboard.store_string(&input)?;

        let output1 = observer1.join().unwrap()?;
        let output2 = observer2.join().unwrap()?;

        assert_eq!(input.len(), output1.len());
        assert_eq!(input, output1);

        assert_eq!(input, output2);
        assert_eq!(input.len(), output2.len());

        println!("drop clipboard");
        drop(clipboard);
        observer3.join().unwrap()?;

        Ok(())
    }
}
