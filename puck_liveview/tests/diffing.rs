#[cfg(feature = "_test_fuzzing")]
mod test_diffing {

    use proptest::{arbitrary::Arbitrary, proptest, strategy::Strategy};
    use puck_liveview::prelude::{BodyNode, IntoWrappedBodyNode};

    proptest! {
        #[test]
        fn pt_test_diffing(before in generate_body_node(), after in generate_body_node()) {
            test_diffing_inner(before, after);
        }
    }

    fn generate_body_node() -> impl Strategy<Value = BodyNode> {
        BodyNode::arbitrary_with((8, 256, 10))
    }

    fn test_diffing_inner(before: BodyNode, after: BodyNode) {
        let before = before.wrap().into_element(vec![0]);
        let expected_after = after.wrap().into_element(vec![0]);

        let cs = before.diff(Some(&expected_after));

        let mut actual_after = before.clone();
        cs.apply(&mut actual_after);

        assert_eq!(actual_after, expected_after);
    }
}
