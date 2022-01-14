use actix_web::{
    middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Debug, Serialize, Deserialize)]
struct MyObj {
    input: String
}

fn string_parser(input: String) -> Vec<String> {
    let bracket_removed = input
        .replace("[", "")
        .replace("],", "")
        .replace("]", "")
        .replace(" ", "")
        .replace("'", "");

    let split_by_comma :Vec<String> = bracket_removed.split(",").map(|c| c.to_owned()).collect();
    split_by_comma
}

fn get_first_and_last(input: Vec<String>) -> (String, String) {
    let mut maps: HashMap<String, i32> = HashMap::new();
    let mut flag = true;
    for item in input {
        let default = if flag {1} else {-1};
        *maps.entry(item).or_insert(0) += default;
        flag = !flag;
    }
    let mut first: String = "".to_string();
    let mut last: String = "".to_string();
    for (k, v) in maps.iter() {
        if *v == 1 {
            first = k.clone();
        }
        if *v == -1 {
            last = k.clone();
        }
    }    
    (first, last)
}

/// This handler uses json extractor
async fn index(item: web::Json<MyObj>) -> HttpResponse {
    println!("model: {:?}", &item);
    let parsed_input = string_parser(item.0.input.clone());
    let (first, last) = get_first_and_last(parsed_input);
    let result : String = format!("{{'{}', '{}'}}", first, last);
    HttpResponse::Ok().json(result) // <- send response
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .data(web::JsonConfig::default().limit(4096)) // <- limit size of the payload (global configuration)
            .service(web::resource("/").route(web::post().to(index)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::dev::Service;
    use actix_web::{http, test, web, App};

    #[actix_rt::test]
    async fn test_index() -> Result<(), Error> {
        let mut app = test::init_service(
            App::new().service(web::resource("/").route(web::post().to(index))),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&MyObj {
                input: "[['IND', 'EWR'], ['SFO', 'ATL'], ['GSO', 'IND'], ['ATL', 'GSO']] ".to_owned(),
            })
            .to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = match resp.response().body().as_ref() {
            Some(actix_web::body::Body::Bytes(bytes)) => bytes,
            _ => panic!("Response error"),
        };

        assert_eq!(response_body, r##"{'SFO', 'EWR'}"##);

        Ok(())
    }

    #[test]
    fn test_string_parse() {
        let test_input1 = "[['SFT', 'EWR']]";
        let test_res1 = string_parser(test_input1.to_owned());
        assert_eq!(test_res1, vec!["SFT", "EWR"]);
    }

    #[test]
    fn test_get_first_and_last() {
        let test_input1 = vec!["SFT".to_owned(), "EWR".to_owned()];
        let (res1, res2) = get_first_and_last(test_input1);
        assert_eq!(res1, "SFT".to_owned());
        assert_eq!(res2, "EWR".to_owned());
    }
}
