use std::collections::HashMap;
use std::marker::PhantomData;


#[derive(Debug, PartialEq, PartialOrd, Eq, Hash, Clone, Copy)]
pub struct LimitedInt<const N: u8>(pub u8, PhantomData<u8>);

impl <const N: u8> LimitedInt<N> {
    pub fn new(value: u8) -> Self {
        return Self(value % N, PhantomData)
    }

    pub fn all_values() -> Vec<Self> { // TODO: This should be an iterator
        let mut output: Vec<Self> = vec![];
        for i in 0..N {
            output.push(Self::new(i));
        }
        return output // return 0..N ????
    }

    pub fn adjacent_values(&self) -> [LimitedInt<N>; 2] {
        let value = self.0;
        let prev = Self::new(value + N - 1);
        let next = Self::new(value + 1);
        [prev, next]
    }

    pub fn map_to_other<const T: u8>() -> HashMap<Self, LimitedInt<T>> {
        let mut output = HashMap::new();
        for i in 0..N {
            let new_value = (
                T as f64 * (
                    1.0 - (
                        i as f64 / N as f64
                    )
                )
            ).round() as u8 % T;
            output.insert(Self::new(i), LimitedInt::<T>::new(new_value));
        }
        return output
    }

    pub fn shift_by(&self, shift: u8) -> Self {
        Self::new(self.0 + shift)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_within_limit() {
        assert_eq!(
            LimitedInt::<6>::new(0),
            LimitedInt::<6>::new(6)
        )
    }

    #[test]
    fn test_adjacent_values() {
        assert_eq!(
            LimitedInt::<6>::new(0).adjacent_values(),
            [LimitedInt::<6>::new(5), LimitedInt::<6>::new(1)]
        )
    }

    #[test]
    fn test_all_values() {
        assert_eq!(
            LimitedInt::<6>::all_values(),
            vec![
                LimitedInt::<6>::new(0),
                LimitedInt::<6>::new(1),
                LimitedInt::<6>::new(2),
                LimitedInt::<6>::new(3),
                LimitedInt::<6>::new(4),
                LimitedInt::<6>::new(5)
            ]
        )
    }

    #[test]
    fn test_map_to_other() {
        let mut result = HashMap::new();
        result.insert(LimitedInt::<6>::new(0), LimitedInt::<10>::new(0));
        result.insert(LimitedInt::<6>::new(1), LimitedInt::<10>::new(8));
        result.insert(LimitedInt::<6>::new(2), LimitedInt::<10>::new(7));
        result.insert(LimitedInt::<6>::new(3), LimitedInt::<10>::new(5));
        result.insert(LimitedInt::<6>::new(4), LimitedInt::<10>::new(3));
        result.insert(LimitedInt::<6>::new(5), LimitedInt::<10>::new(2)); 
        assert_eq!(
            LimitedInt::<6>::map_to_other::<10>(),
            result
        )
    }

    #[test]
    fn test_shift_by() {
        assert_eq!(
            LimitedInt::<6>::new(3).shift_by(4),
            LimitedInt::<6>::new(1)
        )
    }
}

