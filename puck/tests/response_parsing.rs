use std::io::Cursor;

use itertools::Itertools;
use proptest::prelude::*;
use puck::{encoder::Encoder, request::Body, Response};

fn ascii_headers() -> impl Strategy<Value = (Vec<(String, String)>, usize)> {
    // 19 because we add one!
    prop::collection::vec(("[a-zA-Z][a-zA-Z-]+", "[a-zA-Z][a-zA-Z-]+"), 1..19).prop_flat_map(
        |vec| {
            let len = vec.len();
            (Just(vec), 0..len)
        },
    )
}

proptest! {
    #[test]
    #[cfg(feature="_test_fuzzing")]
    fn pt_test_response_parsing(headers in ascii_headers(), body: String) {
        test_http_response_parsing(headers, body);
    }
}

fn test_http_response_parsing(headers: (Vec<(String, String)>, usize), body: String) {
    let mut headers = headers.0;

    let response = Response::build()
        .headers(headers.clone())
        .body(Body::from_string(body.clone()))
        .build();

    let mut encoder = Encoder::new(response);

    let mut buf = vec![];
    encoder
        .write_tcp_stream(&mut buf)
        .expect("failed to encode response");

    let cursor = Cursor::new(buf);

    let mut response = Response::parse(cursor)
        .expect("failed to parse response")
        .expect("blank response");

    let response_body = response
        .take_body()
        .into_string()
        .expect("failed to interpret body as string");

    assert_eq!(response_body, body);

    headers.push((
        "Content-Type".to_string(),
        "application/octet-stream".to_string(),
    ));

    for ((input_key, input_value), (parsed_key, parsed_value)) in response
        .headers()
        .clone()
        .into_iter()
        .sorted()
        .zip(headers.into_iter().sorted())
    {
        assert_eq!(input_key, parsed_key);
        assert_eq!(input_value, parsed_value);
    }
}
