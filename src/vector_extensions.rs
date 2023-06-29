use ndarray::{ArrayBase, Data, Ix1, RawData};

///! This module contains the vector extensions trait

pub trait VectorExtensions {
    /// get the monotonic property of the vector
    fn monotonic_prop(&self) -> Monotonic;
}

/// Describes the monotonic property of a vector
#[derive(Debug)]
pub enum Monotonic {
    Rising { strict: bool },
    Falling { strict: bool },
    NotMonotonic,
}
use Monotonic::*;

impl<S, T> VectorExtensions for ArrayBase<S, Ix1>
where
    S: RawData<Elem = T> + Data,
    T: PartialOrd,
{
    fn monotonic_prop(&self) -> Monotonic {
        if self.len() <= 1 {
            return NotMonotonic;
        };

        #[derive(Debug)]
        enum State {
            Init,
            NotStrict,
            Known(Monotonic),
        }
        use State::*;

        let state = self
            .windows(2)
            .into_iter()
            .try_fold(Init, |state, items| {
                let a = items.get(0).unwrap_or_else(|| unreachable!());
                let b = items.get(1).unwrap_or_else(|| unreachable!());
                match state {
                    Init => {
                        if a < b {
                            return Ok(Known(Rising { strict: true }));
                        } else if a == b {
                            return Ok(NotStrict);
                        }
                        Ok(Known(Falling { strict: true }))
                    }
                    NotStrict => {
                        if a < b {
                            return Ok(Known(Rising { strict: false }));
                        } else if a == b {
                            return Ok(NotStrict);
                        }
                        Ok(Known(Falling { strict: false }))
                    }
                    Known(Rising { strict }) => {
                        if a == b {
                            return Ok(Known(Rising { strict: false }));
                        } else if a < b {
                            return Ok(Known(Rising { strict }));
                        }
                        Err(NotMonotonic)
                    }
                    Known(Falling { strict }) => {
                        if a == b {
                            return Ok(Known(Falling { strict: false }));
                        } else if a > b {
                            return Ok(Known(Falling { strict }));
                        }
                        Err(NotMonotonic)
                    }
                    Known(NotMonotonic) => unreachable!(),
                }
            })
            .unwrap_or(Known(NotMonotonic));

        if let Known(state) = state {
            state
        } else {
            NotMonotonic
        }
    }
}

#[cfg(test)]
mod test {
    use ndarray::{array, s, Array1};

    use super::{Monotonic, VectorExtensions};

    macro_rules! test_monotonic {
        ($d:ident, $expected:pat) => {
            match $d.monotonic_prop() {
                $expected => (),
                value => panic!("{}", format!("got {value:?}")),
            };
            match $d.slice(s![..;1]).monotonic_prop() {
                $expected => (),
                _ => panic!(),
            };
        };
    }

    // test with f64
    #[test]
    fn test_strict_monotonic_rising_f64() {
        let data: Array1<f64> = array![1.1, 2.0, 3.123, 4.5];
        test_monotonic!(data, Monotonic::Rising { strict: true });
    }

    #[test]
    fn test_monotonic_rising_f64() {
        let data: Array1<f64> = array![1.1, 2.0, 3.123, 3.123, 4.5];
        test_monotonic!(data, Monotonic::Rising { strict: false });
    }

    #[test]
    fn test_strict_monotonic_falling_f64() {
        let data: Array1<f64> = array![5.8, 4.123, 3.1, 2.0, 1.0];
        test_monotonic!(data, Monotonic::Falling { strict: true });
    }

    #[test]
    fn test_monotonic_falling_f64() {
        let data: Array1<f64> = array![5.8, 4.123, 3.1, 3.1, 2.0, 1.0];
        test_monotonic!(data, Monotonic::Falling { strict: false });
    }

    #[test]
    fn test_not_monotonic_f64() {
        let data: Array1<f64> = array![1.1, 2.0, 3.123, 3.120, 4.5];
        test_monotonic!(data, Monotonic::NotMonotonic);
    }

    // test with i32
    #[test]
    fn test_strict_monotonic_rising_i32() {
        let data: Array1<i32> = array![1, 2, 3, 4, 5];
        test_monotonic!(data, Monotonic::Rising { strict: true });
    }

    #[test]
    fn test_monotonic_rising_i32() {
        let data: Array1<i32> = array![1, 2, 3, 3, 4, 5];
        test_monotonic!(data, Monotonic::Rising { strict: false });
    }

    #[test]
    fn test_strict_monotonic_falling_i32() {
        let data: Array1<i32> = array![5, 4, 3, 2, 1];
        test_monotonic!(data, Monotonic::Falling { strict: true });
    }

    #[test]
    fn test_monotonic_falling_i32() {
        let data: Array1<i32> = array![5, 4, 3, 3, 2, 1];
        test_monotonic!(data, Monotonic::Falling { strict: false });
    }

    #[test]
    fn test_not_monotonic_i32() {
        let data: Array1<i32> = array![1, 2, 3, 2, 4, 5];
        test_monotonic!(data, Monotonic::NotMonotonic);
    }

    #[test]
    fn test_ordered_view_on_unordred_array() {
        let data: Array1<i32> = array![5, 4, 3, 2, 1];
        let ordered = data.slice(s![..;-1]);
        test_monotonic!(ordered, Monotonic::Rising { strict: true });
    }

    #[test]
    fn test_starting_flat() {
        let data: Array1<i32> = array![1, 1, 2, 3, 4, 5];
        test_monotonic!(data, Monotonic::Rising { strict: false });
    }

    #[test]
    fn test_flat() {
        let data: Array1<i32> = array![1, 1, 1];
        test_monotonic!(data, Monotonic::NotMonotonic);
    }

    #[test]
    fn test_one_element_array() {
        let data: Array1<i32> = array![1];
        test_monotonic!(data, Monotonic::NotMonotonic);
    }
}
