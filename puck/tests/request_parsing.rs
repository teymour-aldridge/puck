#[cfg(feature = "_test_fuzzing")]
mod test {
    use itertools::Itertools;
    use proptest::prelude::*;
    use puck::{
        request::{Body, Method},
        Request,
    };

    fn ascii_headers() -> impl Strategy<Value = (Vec<(String, String)>, usize)> {
        // 19 because we add one!
        prop::collection::vec(
            ("[a-zA-Z][a-zA-Z-]*[a-zA-Z]", "[a-zA-Z][a-zA-Z-]*[a-zA-Z]"),
            1..19,
        )
        .prop_flat_map(|vec| {
            let len = vec.len();
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
        let req = Request::parse(cursor)
            .expect("failed to parse")
            .expect("the request body was empty");
        let parsed_body_string = req.body.into_string().expect("failed to parse body");
        assert_eq!(parsed_body_string, body);
        for ((input_key, input_value), (parsed_key, parsed_value)) in req
            .headers
            .into_iter()
            .sorted()
            .zip(headers.into_iter().sorted())
        {
            assert_eq!(input_key, parsed_key);
            assert_eq!(input_value, parsed_value);
        }
    }
}
