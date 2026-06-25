#[derive(Debug, Clone, Default)]
pub struct DownloadProgress {
    downloaded: u64,
    total: Option<u64>,
}

impl DownloadProgress {
    pub fn new(total: Option<u64>) -> Self {
        Self {
            downloaded: 0,
            total,
        }
    }

    pub fn advance(&mut self, bytes: u64) {
        self.downloaded += bytes;
    }

    pub fn downloaded(&self) -> u64 {
        self.downloaded
    }

    pub fn total(&self) -> Option<u64> {
        self.total
    }
}
