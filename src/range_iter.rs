use reqwest::header::HeaderValue;

pub struct RangeIter {
    start: u64,
    end: u64,
    buffer_size: u64,
    check: bool,
}

impl RangeIter {
    pub fn new(start: u64, end: u64, buffer_size: u64, check: bool) -> Self {
        RangeIter {
            start,
            end,
            buffer_size,
            check,
        }
    }
}

impl Iterator for RangeIter {
    type Item = (HeaderValue, u64, u64);
    fn next(&mut self) -> Option<Self::Item> {
        if self.start > self.end {
            None
        } else {
            let prev_start = self.start;

            if !self.check && self.end - self.start > self.buffer_size && self.end - self.start < 2 * self.buffer_size {
                self.start = self.end + 1;
                Some((HeaderValue::from_str(&format!("bytes={}-{}", prev_start, self.end)).unwrap(), prev_start, self.end))
            } else {
                self.start += std::cmp::min(self.buffer_size as u64, self.end - self.start + 1);
                Some((HeaderValue::from_str(&format!("bytes={}-{}", prev_start, self.start - 1)).unwrap(), prev_start, self.start - 1))
            }
        }
    }
}