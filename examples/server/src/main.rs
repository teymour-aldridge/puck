use puck::{
    app::{App, Handler, Route},
    at,
    body::Body,
    run_app, Response,
};

fn main() {
    run_app!(
        App::new().route(Route::new(
            at!("hello" => "world"),
            Handler::Http(|_, _| {
                Response::build()
                    .body(Body::from_string("<h1>Hello World!</h1>"))
                    .build()
            })
        ),),
        "127.0.0.1:5050",
        ()
    );
}
