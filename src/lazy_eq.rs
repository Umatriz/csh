pub trait LazyEq<Rhs: ?Sized = Self> {
    fn lazy_eq(&self, other: &Rhs) -> bool;

    fn lazy_ne(&self, other: &Rhs) -> bool {
        !self.lazy_eq(other)
    }
}

impl<T: Eq> LazyEq for Vec<T> {
    fn lazy_eq(&self, other: &Self) -> bool {
        self.len() == other.len()
            && 'check: {
                for el in other {
                    if !self.contains(el) {
                        break 'check false;
                    }
                }
                true
            }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lazy_eq_test_1() {
        let vec1 = vec![1, 3, 2];
        let vec2 = vec![3, 1, 2];
        assert!(vec1.lazy_eq(&vec2));
    }

    #[test]
    fn lazy_ne_test_1() {
        let vec1 = vec![1];
        let vec2 = vec![3, 1, 2];
        assert!(vec1.lazy_ne(&vec2));
    }

    #[test]
    fn lazy_ne_test_2() {
        let vec1 = vec![8, 5, 13, 8];
        let vec2 = vec![3, 1, 2];
        assert!(vec1.lazy_ne(&vec2));
    }

    #[test]
    fn lazy_ne_test_3() {
        let vec1 = vec![8, 5, 13];
        let vec2 = vec![3, 1, 2];
        assert!(vec1.lazy_ne(&vec2));
    }

    #[test]
    fn lazy_ne_test_4() {
        let vec1 = vec![0, 0, 0];
        let vec2 = vec![3, 1, 2];
        assert!(vec1.lazy_ne(&vec2));
    }
}
