use malvolio::prelude::*;

use crate::prelude::*;

#[test]
fn diffing_regression_2021_06_06() {
    let mut before = H1::new("")
        .raw_attribute("¡", "")
        .wrap()
        .into_element(vec![0]);
    let expected_after = H1::new("").wrap().into_element(vec![0]);

    let diff_before = before.clone();

    let cs = diff_before.diff(Some(&expected_after));

    cs.apply(&mut before);

    assert_eq!(before, expected_after);
}
