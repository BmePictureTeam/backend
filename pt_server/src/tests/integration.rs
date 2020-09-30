// TODO
// use crate::{
//     config::Config,
//     server::routes::{setup_routes, HelloResponse},
// };
// use actix_web::{test, App};

// #[actix_rt::test]
// async fn test_hello() {
//     let config = Config::from_env().unwrap();
//     let mut app = test::init_service(App::new().data(config).configure(setup_routes)).await;
//     let req = test::TestRequest::get()
//         .uri("/hello?name=jozsi")
//         .to_request();

//     let hello_resp: HelloResponse = test::read_response_json(&mut app, req).await;
//     assert!(hello_resp.message.contains("jozsi"));
// }
