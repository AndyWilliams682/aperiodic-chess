pub trait LimitedIntTrait {
    fn new(value: u8) -> Option<Self> where Self: Sized;
    fn max_value() -> u8;
    fn all_values() -> Vec<Self> where Self: Sized;
    fn adjacent_values(&self) -> Vec<Self> where Self: Sized;
}

#[macro_export]
macro_rules! create_limited_int {
    ($name:ident, $max_value:expr) => {
        #[derive(Debug, PartialEq, PartialOrd)]
        struct $name(u8); // May need to modify u8 in the future?

        impl $name {
            fn new_internal(value: u8) -> Self {
                Self(value)
            }
           
            pub fn get_map_to_other<T: LimitedIntTrait>() -> Vec<u8> {
                let mut output = vec![];
                let t_max = T::max_value();
                let self_max = Self::max_value();
               
                for n in 0..self_max {
                    let new_value = (
                        t_max as f64 * (
                            1.0 - (
                                n as f64 / self_max as f64
                            )
                        )
                    ).round() as u8 % t_max;
                    output.push(new_value) // TODO: This should return the other type T
                }
                return output
            } // Replace 1 with 0.5 to get the backwards mapping, same as shifting by the mid value?
            // Might be able to use the exact same mapping to encode forwards and backwards
        }
       
        impl LimitedIntTrait for $name {
            fn new(value: u8) -> Option<Self> {
                if value < $max_value {
                    Some(Self(value))
                } else {
                    None
                }
            }

            fn max_value() -> u8 {
                $max_value
            }
           
            fn all_values() -> Vec<Self> { // TODO: This should be an iterator
                let mut output: Vec<Self> = vec![];
                for n in 0..Self::max_value() {
                    output.push(Self::new_internal(n));
                }
                return output
            }
           
            fn adjacent_values(&self) -> Vec<Self> {
                let value = self.0;
                let max_value = $max_value;
                let prev = Self::new_internal((value + max_value - 1) % max_value);
                let next = Self::new_internal((value + 1) % max_value);
                vec![prev, next]
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    create_limited_int!(LimitedInt6, 6);
    create_limited_int!(LimitedInt10, 10);

    #[test]
    fn test_new_within_limit() {
        assert_eq!(
            LimitedInt6::new(5).unwrap(),
            LimitedInt6::new_internal(5)
        )
    }

    #[test]
    #[should_panic]
    fn test_new_beyond_limit() {
        LimitedInt6::new(6).unwrap();
    }

    #[test]
    fn test_adjacent_values() {
        assert_eq!(
            LimitedInt6::new_internal(0).adjacent_values(),
            vec![
                LimitedInt6::new_internal(5),
                LimitedInt6::new_internal(1)
            ]
        )
    }

    #[test]
    fn test_max_value() {
        assert_eq!(
            LimitedInt6::max_value(),
            6
        )
    }

    #[test]
    fn test_all_values() {
        assert_eq!(
            LimitedInt6::all_values(),
            vec![
                LimitedInt6::new_internal(0),
                LimitedInt6::new_internal(1),
                LimitedInt6::new_internal(2),
                LimitedInt6::new_internal(3),
                LimitedInt6::new_internal(4),
                LimitedInt6::new_internal(5)
            ]
        )
    }

    #[test]
    fn test_get_map_to_other() {
        assert_eq!(
            LimitedInt6::get_map_to_other::<LimitedInt10>(),
            vec![0, 8, 7, 5, 3, 2]
        )
    }
}

