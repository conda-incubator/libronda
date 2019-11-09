// some helpful discussion on how this works is at https://github.com/la10736/rstest/issues/66

/// Paste identifiers within a macro invocation that expands to one or more
/// macro_rules macros or items containing macros.

macro_rules! parametrize_versions {
        ( $test:ident ) => {
            paste::item! {
            #[rstest_parametrize(v_string, n_parts,
            case::one_int("1", 1),
            case::two_ints("1.2", 2),
            case::four_ints("1.2.3.4", 4),
            case::eight_ints("1.2.3.4.5.6.7.8", 8),
            case::zero("0", 1),
            case::three_zeros("0.0.0", 3),
            case::trailing_zeros_kept("1.0.0", 3),
            case::leading_zeros_kept("0.0.1", 3),
            case::empty_string("", 1),
            case::only_period(".", 1),
            case::only_three_periods("...", 1),
            // 2dev is considered one piece
            case::dev_post_version("1.2dev", 2),
            // 2.dev is considered two pieces (and dev has an implicit leading zero and trailing zero)
            case::dev_post_version_with_dot("1.2.dev", 3),
            case::dev_post_version_with_dot_and_post_rev("1.2.dev2", 3),
            case::post_rev_after_period("1.2.alpha.4", 4),
            )]
            fn [< _ $test >] (v_string: &str, n_parts: usize) {
                $test(v_string, n_parts)
            }
        }
    }
}

//macro_rules! parametrize_versions_errors {
//    ( $test:ident ) => {
//        paste::item! {
//            #[rstest_parametrize(v_string, n_parts,
//            // case("abc", 1),
//            // case("alpha.dev.snapshot", 3),
//            case("test. .snapshot", 3),
//            // TODO: broken case, decide what to do here
//            // case("$", 1),
//            )]
//            fn [< _ $test >] (v_string: &str, n_parts: usize) {
//                $test(v_string, n_parts)
//            }
//        }
//    }
//}

/// List of version sets for dynamic tests
macro_rules! parametrize_versions_set {
    ( $test:ident ) => {
        paste::item! {
            #[rstest_parametrize(a, b, operator,
            case::equal_strings("1", "1", &CompOp::Eq),
            case::more_zeros_left("1.0.0.0", "1", &CompOp::Eq),
            case::more_zeros_right("1", "1.0.0.0", &CompOp::Eq),
            case::equal_zeros("0", "0", &CompOp::Eq),
            case::zero_more_zeros_left("0.0.0", "0", &CompOp::Eq),
            case::zero_more_zeros_left("0", "0.0.0", &CompOp::Eq),
            case::blank_eq_blank("", "", &CompOp::Eq),
            // Not true for conda - empty string is its own special type that is lowest priority
            // case::blank_eq_zero("", "0.0", &CompOp::Eq),
            // case::zero_eq_blank("0.0", "", &CompOp::Eq),
            case::blank_lt_0_1("", "0.1", &CompOp::Lt),
            case::_0_1_gt_blank("0.1", "", &CompOp::Gt),
            case::bugfix_increment("1.2.3", "1.2.4", &CompOp::Lt),
            case::fourth_place_greater("1.0.0.1", "1.0.0.0", &CompOp::Gt),
            case::fourth_place_lower("1.0.0.0", "1.0.0.1", &CompOp::Lt),
            case::left_more_specific("1.2.3.4", "1.2", &CompOp::Gt),
            case::right_more_specific("1.2", "1.2.3.4", &CompOp::Lt),
            case::specific_less_than_major_ver("1.2.3.4", "2", &CompOp::Lt),
            case::major_ver_beats_specific("2", "1.2.3.4", &CompOp::Gt),
            case::periods_split("123", "1.2.3", &CompOp::Gt),
            case::dev_version_gtr_than_previous_release("1.1.2", "1.1.30-dev", &CompOp::Lt),
            case::alpha_version_lower_than_same_release("1.2.3", "1.2.3.alpha", &CompOp::Gt),
            case::dev_version_lower_than_same_release("1.2.3", "1.2.3-dev", &CompOp::Gt),
            // lexicographic sorting makes dev higher than alpha.  Other version part implementations
            //    may change this in their comparison implementations.
            case::dev_version_lss_than_alpha_dots("1.2.3.dev", "1.2.3.alpha", &CompOp::Lt),
            case::dev_version_lss_than_alpha_dashes("1.2.3dev", "1.2.3alpha", &CompOp::Lt),
            case::lexicographical_post_version("1.2.3d", "1.2.3a", &CompOp::Gt),
            case::dev_version_lss_than_alpha_dots_post_dev("1.2.3.dev.1", "1.2.3.alpha", &CompOp::Lt),
            // case::full_string_compared_with_short("version-compare 3.2.0 / build 0932", "3.2.5", &CompOp::Lt),
            // case::full_string_compared_with_short_gtr("version-compare 3.2.0 / build 0932", "3.1.1", &CompOp::Gt),
//            case::full_string_eq(
//                "version-compare 1.4.1 / build 0043",
//                "version-compare 1.4.1 / build 0043",
//                &CompOp::Eq,
//            ),
//            case::full_string_build_number_increment(
//                "version-compare 1.4.1 / build 0042",
//                "version-compare 1.4.1 / build 0043",
//                &CompOp::Lt,
//            ),
//            // TODO: inspect these cases
//            case::snapshot_lt_alpha_dot("snapshot.1.2.3", "1.2.3.alpha", &CompOp::Lt),
//            case::snapshot_lt_alpha_dash("snapshot-1.2.3", "1.2.3-alpha", &CompOp::Lt),
            )]
            fn [< _ $test>] (a: &str, b: &str, operator: &CompOp) {
                $test(a, b, operator)
            }
        }
    }
}

/// List of invalid version sets for dynamic tests
macro_rules! parametrize_errors_set {
    ( $test:ident ) => {
        paste::item! {
            #[rstest_parametrize(a, b, operator,
            case::wrong_operator_lt("1.2.3", "1.2.3", &CompOp::Lt),
            case::wrong_operator_ne("1.2", "1.2.0.0", &CompOp::Ne),
            case::dev_alone_is_not_eq("1.2.3.dev", "dev", &CompOp::Eq),
            // not an error, conda considers alpha lower than numbers
            //case("snapshot", "1", &CompOp::Lt),
            )]
            fn [< _ $test>] (a: &str, b: &str, operator: &CompOp) {
                $test(a, b, operator)
            }
        }
    }
}