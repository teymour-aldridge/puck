#[cfg(feature = "_test_fuzzing")]
mod test {
    use std::collections::HashMap;

    use proptest::prelude::*;
    use puck::{body::Body, request::Method, Request};

    fn ascii_headers() -> impl Strategy<Value = (Vec<(String, String)>, usize)> {
        // 19 because we add one!
        prop::collection::vec(
            ("[a-zA-Z][a-zA-Z-]*[a-zA-Z]", "[a-zA-Z][a-zA-Z-]*[a-zA-Z]"),
            1..19,
        )
        .prop_flat_map(|vec| {
            let len = vec.len();
            let map = vec.into_iter().collect::<HashMap<String, String>>();
            let vec = map.into_iter().collect::<Vec<(String, String)>>();
            (Just(vec), 0..len)
        })
    }

    proptest! {
        #[test]
        #[cfg(feature="_test_fuzzing")]
        fn pt_request_parsing(headers in ascii_headers(), body: String) {
            test_http_request_parsing(headers, body)
        }
    }

    fn test_http_request_parsing(headers: (Vec<(String, String)>, usize), body: String) {
        let mut headers = headers.0;
        headers.push(("Host".to_string(), "example.com".to_string()));
        let mut req = Request::build("http://example.com/")
            .headers(headers.clone())
            .method(Method::Get)
            .body(Body::from_string(body.clone()))
            .build();

        let mut buffer = vec![];
        req.write(&mut buffer)
            .expect("failed to write request into buffer");
        let cursor = std::io::Cursor::new(buffer);
        let mut req = Request::parse(cursor)
            .expect("failed to parse")
            .expect("the request body was empty");
        let parsed_body_string = req.take_body().into_string().expect("failed to parse body");
        assert_eq!(parsed_body_string, body);
        for (a, b) in req.headers() {
            if a != "Content-Type" {
                assert!(headers.contains(&(a.clone(), b.clone())));
            }
        }
    }
}
