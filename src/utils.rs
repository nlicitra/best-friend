#[allow(dead_code)]
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn mean(set: &[f32]) -> f32 {
    set.iter().sum::<f32>() / (set.len() as f32)
}

pub fn median(set: &[f32]) -> f32 {
    let mut copy = vec![0.; set.len()];
    copy[..].clone_from_slice(set);
    copy.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let middle_index = copy.len() / 2;
    if copy.len() % 2 == 0 {
        return mean(&copy[middle_index - 1..middle_index]);
    }
    copy[middle_index]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean() {
        let set: &[f32] = &[1., 2., 3., 4.];

        assert_eq!(mean(set), 2.5);
    }

    #[test]
    fn test_median_even() {
        let set: &[f32] = &[1., 2., 3., 4.];

        assert_eq!(mean(set), 2.5);
    }

    #[test]
    fn test_median_odd() {
        let set: &[f32] = &[1., 2., 3.];

        assert_eq!(mean(set), 2.);
    }

    #[test]
    fn test_median_one() {
        let set: &[f32] = &[1.];

        assert_eq!(mean(set), 1.);
    }
}
