use std::io::Cursor;

use crate::{body::Body, request::Method, Request};

fn execute_test(headers: Vec<(String, String)>, body: impl ToString) {
    let mut req = Request::build("http://example.com")
        .headers(headers.clone())
        .method(Method::Get)
        .body(Body::from_string(body))
        .build();

    let mut vec = Vec::new();

    req.write(&mut vec).expect("failed to write request");

    let req = Request::parse(Cursor::new(vec))
        .expect("failed to parse request")
        .expect("emtpy request");

    for header in req.headers {
        assert!(headers.contains(&header))
    }
}

#[lunatic::test]
/// This regression test came from https://github.com/bailion/puck/runs/2756775397
fn test_inverse_request_regression_2021_06_06_morning() {
    let headers = vec![
        ("NW".to_string(), "gcfZ-spKEf-v-gh".to_string()),
        ("Host".to_string(), "example.com".to_string()),
    ];
    execute_test(headers, "u𘐿\\K;ῗ𐰑𖭜R");
}

#[lunatic::test]
/// This regression test came from https://github.com/bailion/puck/runs/2758615118
fn test_inverse_request_regression_2021_06_06_afternoon() {
    let headers = vec![
        ("aA".to_string(), "aa".to_string()),
        ("Host".to_string(), "example.com".to_string()),
    ];
    execute_test(headers, "");
}
