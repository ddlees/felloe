use indicatif::ProgressBar;
use std::{
    io::{self, Read},
    sync::Arc,
};

pub struct DownloadProgress<R> {
    pub stream: R,
    pub pb: Arc<ProgressBar>,
}

impl<R: Read> Read for DownloadProgress<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf).map(|n| {
            self.pb.inc(n as u64);
            n
        })
    }
}
