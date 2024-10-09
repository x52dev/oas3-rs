macro_rules! valid_vs_invalid {
    ($validator:expr, $valid:expr, $invalid:expr,) => {{
        use crate::validation::Path;

        let valid: &[&JsonValue] = $valid;
        let invalid: &[&JsonValue] = $invalid;

        for item in valid {
            log::trace!("should be Ok {:?}", &item);
            assert!($validator.validate(&item, Path::default()).is_ok())
        }

        for item in invalid {
            log::trace!("should be Err {:?}", &item);
            assert!($validator.validate(&item, Path::default()).is_err())
        }
    }};

    ($validator:expr, $valid:expr, $invalid:expr) => {{
        valid_vs_invalid!($validator, $valid, $invalid,)
    }};
}
