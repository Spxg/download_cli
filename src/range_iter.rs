use reqwest::header::HeaderValue;

pub struct RangeIter {
    start: u64,
    end: u64,
    buffer_size: u64,
    task_count: u64,
    now_at: u64,
}

impl RangeIter {
    pub fn new(start: u64, end: u64, task_count: u64) -> Self {
        let now_at = 0;
        let buffer_size = (end - start) / task_count;
        RangeIter {
            start,
            end,
            buffer_size,
            task_count,
            now_at,
        }
    }
}

impl Iterator for RangeIter {
    type Item = (HeaderValue, u64, u64);
    fn next(&mut self) -> Option<Self::Item> {
        if self.now_at >= self.task_count {
            None
        } else {
            let prev_start = self.start;
            self.now_at += 1;
            if self.now_at == self.task_count {
                Some((HeaderValue::from_str(&format!("bytes={}-{}", prev_start, self.end)).unwrap(), prev_start, self.end))
            } else {
                self.start += self.buffer_size as u64;
                Some((HeaderValue::from_str(&format!("bytes={}-{}", prev_start, self.start - 1)).unwrap(), prev_start, self.start - 1))
            }
        }
    }
}