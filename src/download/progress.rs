use indicatif::{ProgressBar, ProgressStyle};

use crate::utils::logging;

#[derive(Debug, Clone, Default)]
pub struct DownloadProgress {
    downloaded: u64,
    total: Option<u64>,
    bar: Option<ProgressBar>,
}

impl DownloadProgress {
    pub fn new(total: Option<u64>) -> Self {
        let bar = if logging::quiet() {
            None
        } else {
            let bar = match total {
                Some(total) => ProgressBar::new(total),
                None => ProgressBar::new_spinner(),
            };
            bar.set_style(progress_style());
            bar.set_message("downloading");
            Some(bar)
        };

        Self {
            downloaded: 0,
            total,
            bar,
        }
    }

    pub fn advance(&mut self, bytes: u64) {
        self.downloaded += bytes;
        if let Some(bar) = &self.bar {
            bar.inc(bytes);
        }
    }

    pub fn finish(&self) {
        if let Some(bar) = &self.bar {
            bar.finish_with_message("downloaded");
        }
    }

    pub fn downloaded(&self) -> u64 {
        self.downloaded
    }

    pub fn total(&self) -> Option<u64> {
        self.total
    }
}

fn progress_style() -> ProgressStyle {
    ProgressStyle::with_template(
        "{spinner:.green} {msg} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes}",
    )
    .unwrap_or_else(|_| ProgressStyle::default_bar())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_downloaded_bytes() {
        let mut progress = DownloadProgress::default();

        progress.advance(10);
        progress.advance(5);

        assert_eq!(progress.downloaded(), 15);
        assert_eq!(progress.total(), None);
    }
}
