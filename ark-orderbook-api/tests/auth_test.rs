use actix_web::test;
use actix_web_httpauth::extractors::basic::BasicAuth;
use actix_web_httpauth::headers::authorization::Basic;
use ark_orderbook_api::routes::auth::validator;

#[actix_rt::test]
async fn test_validator() {
    // Set environment variables
    std::env::set_var("API_USER", "user");
    std::env::set_var("API_PASSWORD", "password");

    let req = test::TestRequest::default().to_srv_request();

    let basic = Basic::new("wrong_user", Some("wrong_password"));
    let credentials = BasicAuth::from(basic);

    let result = validator(req, credentials).await;
    assert!(result.is_err());

    // Test with correct credentials
    let req = test::TestRequest::default().to_srv_request();
    let correct_user = "user".to_string();
    let correct_password = "password".to_string();

    let basic = Basic::new(correct_user, Some(correct_password));
    let credentials = BasicAuth::from(basic);
    let result = validator(req, credentials).await;
    assert!(result.is_ok());
}
