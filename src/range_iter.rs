use reqwest::header::HeaderValue;

pub struct RangeIter {
    start: u64,
    end: u64,
    buffer_size: u64,
}

impl RangeIter {
    pub fn new(start: u64, end: u64, buffer_size: u64) -> Self {
        RangeIter {
            start,
            end,
            buffer_size,
        }
    }
}

impl Iterator for RangeIter {
    type Item = (HeaderValue, u64);
    fn next(&mut self) -> Option<Self::Item> {
        if self.start > self.end {
            None
        } else {
            let prev_start = self.start;
            self.start += std::cmp::min(self.buffer_size as u64, self.end - self.start + 1);
            Some((HeaderValue::from_str(&format!("bytes={}-{}", prev_start, self.start - 1)).unwrap(), prev_start))
        }
    }
}