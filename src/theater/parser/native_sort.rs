//! Go 1.26 sort.Slice ordering for reference comparators with equal keys.
//! Derived from Go's sort/zsortfunc.go. Copyright 2022 The Go Authors.
//! BSD license retained in docs/GO_LICENSE.txt.
use std::cmp::Ordering;
pub(super) fn sort_by<T>(d: &mut [T], cmp: fn(&T, &T) -> Ordering) {
    let n = d.len();
    if n < 2 {
        return;
    }
    let limit = usize::BITS - n.leading_zeros();
    Sort { d, cmp }.pdq(0, n, limit);
}
struct Sort<'a, T> {
    d: &'a mut [T],
    cmp: fn(&T, &T) -> Ordering,
}
impl<T> Sort<'_, T> {
    fn less(&self, a: usize, b: usize) -> bool {
        (self.cmp)(&self.d[a], &self.d[b]) == Ordering::Less
    }
    fn insertion(&mut self, a: usize, b: usize) {
        for i in a + 1..b {
            let mut j = i;
            while j > a && self.less(j, j - 1) {
                self.d.swap(j, j - 1);
                j -= 1;
            }
        }
    }
    fn sift(&mut self, mut root: usize, hi: usize, first: usize) {
        loop {
            let mut child = 2 * root + 1;
            if child >= hi {
                break;
            }
            if child + 1 < hi && self.less(first + child, first + child + 1) {
                child += 1;
            }
            if !self.less(first + root, first + child) {
                return;
            }
            self.d.swap(first + root, first + child);
            root = child;
        }
    }
    fn heap(&mut self, a: usize, b: usize) {
        let hi = b - a;
        for i in (0..=(hi - 1) / 2).rev() {
            self.sift(i, hi, a);
        }
        for i in (0..hi).rev() {
            self.d.swap(a, a + i);
            self.sift(0, i, a);
        }
    }
    fn order(&self, a: usize, b: usize, swaps: &mut usize) -> (usize, usize) {
        if self.less(b, a) {
            *swaps += 1;
            (b, a)
        } else {
            (a, b)
        }
    }
    fn median(&self, a: usize, b: usize, c: usize, swaps: &mut usize) -> usize {
        let (a, b) = self.order(a, b, swaps);
        let (b, _) = self.order(b, c, swaps);
        self.order(a, b, swaps).1
    }
    fn pivot(&self, a: usize, b: usize) -> (usize, i32) {
        let l = b - a;
        let (mut i, mut j, mut k) = (a + l / 4, a + l / 4 * 2, a + l / 4 * 3);
        let mut swaps = 0;
        if l >= 8 {
            if l >= 50 {
                i = self.median(i - 1, i, i + 1, &mut swaps);
                j = self.median(j - 1, j, j + 1, &mut swaps);
                k = self.median(k - 1, k, k + 1, &mut swaps);
            }
            j = self.median(i, j, k, &mut swaps);
        }
        (
            j,
            match swaps {
                0 => 1,
                12 => -1,
                _ => 0,
            },
        )
    }
    fn partial(&mut self, a: usize, b: usize) -> bool {
        let mut i = a + 1;
        for _ in 0..5 {
            while i < b && !self.less(i, i - 1) {
                i += 1;
            }
            if i == b {
                return true;
            }
            if b - a < 50 {
                return false;
            }
            self.d.swap(i, i - 1);
            if i - a >= 2 {
                let mut j = i - 1;
                while j >= 1 {
                    if !self.less(j, j - 1) {
                        break;
                    }
                    self.d.swap(j, j - 1);
                    j -= 1;
                }
            }
            if b - i >= 2 {
                for j in i + 1..b {
                    if !self.less(j, j - 1) {
                        break;
                    }
                    self.d.swap(j, j - 1);
                }
            }
        }
        false
    }
    fn patterns(&mut self, a: usize, b: usize) {
        let n = b - a;
        if n < 8 {
            return;
        }
        let mut random = n as u64;
        let modulus = 1u64 << (usize::BITS - n.leading_zeros());
        for idx in a + n / 4 * 2 - 1..=a + n / 4 * 2 + 1 {
            random ^= random << 13;
            random ^= random >> 7;
            random ^= random << 17;
            let mut other = (random & (modulus - 1)) as usize;
            if other >= n {
                other -= n;
            }
            self.d.swap(idx, a + other);
        }
    }
    fn equal(&mut self, a: usize, b: usize, pivot: usize) -> usize {
        self.d.swap(a, pivot);
        let (mut i, mut j) = (a + 1, b - 1);
        loop {
            while i <= j && !self.less(a, i) {
                i += 1;
            }
            while i <= j && self.less(a, j) {
                j -= 1;
            }
            if i > j {
                break;
            }
            self.d.swap(i, j);
            i += 1;
            j -= 1;
        }
        i
    }
    fn partition(&mut self, a: usize, b: usize, pivot: usize) -> (usize, bool) {
        self.d.swap(a, pivot);
        let (mut i, mut j) = (a + 1, b - 1);
        while i <= j && self.less(i, a) {
            i += 1;
        }
        while i <= j && !self.less(j, a) {
            j -= 1;
        }
        if i > j {
            self.d.swap(j, a);
            return (j, true);
        }
        self.d.swap(i, j);
        i += 1;
        j -= 1;
        loop {
            while i <= j && self.less(i, a) {
                i += 1;
            }
            while i <= j && !self.less(j, a) {
                j -= 1;
            }
            if i > j {
                break;
            }
            self.d.swap(i, j);
            i += 1;
            j -= 1;
        }
        self.d.swap(j, a);
        (j, false)
    }
    fn pdq(&mut self, mut a: usize, mut b: usize, mut limit: u32) {
        let (mut balanced, mut partitioned) = (true, true);
        loop {
            let len = b - a;
            if len <= 12 {
                self.insertion(a, b);
                return;
            }
            if limit == 0 {
                self.heap(a, b);
                return;
            }
            if !balanced {
                self.patterns(a, b);
                limit -= 1;
            }
            let (mut pivot, mut hint) = self.pivot(a, b);
            if hint == -1 {
                self.d[a..b].reverse();
                pivot = b - 1 - (pivot - a);
                hint = 1;
            }
            if balanced && partitioned && hint == 1 && self.partial(a, b) {
                return;
            }
            if a > 0 && !self.less(a - 1, pivot) {
                a = self.equal(a, b, pivot);
                continue;
            }
            let (mid, already) = self.partition(a, b, pivot);
            partitioned = already;
            let (left, right) = (mid - a, b - mid);
            if left < right {
                balanced = left >= len / 8;
                self.pdq(a, mid, limit);
                a = mid + 1;
            } else {
                balanced = right >= len / 8;
                self.pdq(mid + 1, b, limit);
                b = mid;
            }
        }
    }
}
