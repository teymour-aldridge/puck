use malvolio::prelude::*;

use crate::prelude::*;

use crate::html::id::IdGen;

#[test]
fn diffing_regression_2021_06_06() {
    let mut before = H1::new("")
        .raw_attribute("¡", "")
        .wrap()
        .into_element(&mut IdGen::new());
    let expected_after = H1::new("").wrap().into_element(&mut IdGen::new());

    let diff_before = before.clone();

    let cs = diff_before.diff(Some(&expected_after));

    cs.apply(&mut before);

    assert_eq!(before, expected_after);
}
