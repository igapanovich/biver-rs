pub trait CountIsAtLeast {
    fn count_is_at_least(&mut self, n: usize) -> bool;
}

impl<T: Iterator> CountIsAtLeast for T {
    fn count_is_at_least(&mut self, n: usize) -> bool {
        if n == 0 {
            return true;
        }

        let mut count = 0;
        for _ in self {
            count += 1;
            if count >= n {
                return true;
            }
        }

        false
    }
}
