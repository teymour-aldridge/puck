#[cfg(feature = "_fuzz")]
mod test {
    use fuzzcheck::fuzz_test;

    use puck_liveview::prelude::{BodyNode, IntoWrappedBodyNode};

    #[allow(unused)]
    fn test_diff((before, after): &(BodyNode, BodyNode)) -> bool {
        let before = before.clone();
        let after = after.clone();
        let before = before.wrap().into_element(vec![0]);
        let expected_after = after.wrap().into_element(vec![0]);

        let cs = before.diff(Some(&expected_after));

        let mut actual_after = before.clone();
        cs.apply(&mut actual_after);

        actual_after == expected_after
    }

    #[test]
    fn fuzz_diffing() {
        let res = fuzz_test(test_diff).default_options().launch();
        assert!(!res.found_test_failure);
    }

    fn test_regression(data: &str) {
        let data: (BodyNode, BodyNode) = serde_json::from_str(data).unwrap();

        assert!(test_diff(&data));
    }

    #[test]
    fn regression_1() {
        test_regression(r#"[{"NoScript":{"text":""}},{"Text":{"text":"","attrs":{}}}]"#)
    }

    #[test]
    fn regression_2() {
        test_regression(r#"[{"Text":{"text":"","attrs":{}}},{"Input":{"attrs":{}}}]"#)
    }

    #[test]
    fn many_cov_hits() {
        test_regression(r#"[{"A":{"attrs":{},"text":""}},{"Label":{"text":"","attrs":{}}}]"#);
        test_regression(
            r#"[{"Img":{"attrs":{"9":"6","p":"","F":""}}},{"H2":{"text":"","attrs":{"p":"","C":"","9":""}}}]"#,
        );
    }
}
