use std::io::Cursor;

use crate::{
    request::{Body, Method},
    Request,
};

#[test]
/// This regression test came from https://github.com/bailion/puck/runs/2756775397
fn test_inverse_request() {
    let headers = vec![
        ("NW".to_string(), "gcfZ-spKEf-v-gh".to_string()),
        ("Host".to_string(), "example.com".to_string()),
    ];

    let mut req = Request::build("http://example.com")
        .headers(headers.clone())
        .method(Method::Get)
        .body(Body::from_string("u𘐿\\K;ῗ𐰑𖭜R"))
        .build();

    let mut vec = Vec::new();

    req.write(&mut vec).expect("failed to write request");

    let req = Request::parse(Cursor::new(vec))
        .expect("failed to parse request")
        .expect("emtpy request");

    for header in req.headers {
        dbg!(&header);
        dbg!(&headers);
        assert!(headers.contains(&header))
    }
}
