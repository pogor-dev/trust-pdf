pub fn compute_line_starts(bytes: &[u8]) -> Vec<usize> {
    let mut starts = vec![0usize];
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                starts.push(i + 1);
            }
            b'\r' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                    starts.push(i + 2);
                    i += 1;
                } else {
                    starts.push(i + 1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    starts
}

pub fn offset_to_line_col(offset: usize, line_starts: &[usize]) -> (u32, u32) {
    let mut lo = 0usize;
    let mut hi = line_starts.len();
    while lo + 1 < hi {
        let mid = (lo + hi) / 2;
        if line_starts[mid] <= offset {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let line = lo as u32;
    let col = (offset - line_starts[lo]) as u32;
    (line, col)
}
