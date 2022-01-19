#![allow(unused_imports)]
#[macro_use]
extern crate json;
extern crate serde_derive;

use std::cell::Cell;
use std::fs;
use std::io::Write;
use std::error::Error;
use std::fs::File;
use std::path::Path;

use actix_files as actfs;
use actix_multipart::{Field, Multipart, MultipartError};
use actix_web::{error, middleware, web, App, Error as ActixError, HttpRequest, HttpResponse, HttpServer};
use futures::future::{err, Either};
use futures::{Future, Stream};
use serde_json::{Result, json, Value};
use serde::Serialize;
use serde::Deserialize;

pub struct AppState {
    pub counter: Cell<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ViewContent {
    pub totalcnt: i32,
    pub r#loop: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    app_path: String,
    ip_port_set: String,
}

fn load_ip_port() -> String {
    let path = Path::new("config.json");
    if path.exists() {
        let config: Config = serde_json::from_value(file_read_to_json("config.json").unwrap()).unwrap();
        config.ip_port_set
    } else {
        "0.0.0.0:9999".to_string()
    }
}

fn load_app_path() -> String {
    let path = Path::new("config.json");
    if path.exists() {
        let config: Config = serde_json::from_value(file_read_to_json("config.json").unwrap()).unwrap();
        config.app_path
    } else {
        "http://172.31.50.155:9999/".to_string()
    }
}

pub fn json_re_parse(_res: &str) -> Value {
    json!({ "result": _res })
}

pub fn res_error() -> actix_web::HttpResponse {
    actix_web::HttpResponse::MethodNotAllowed().json(json_re_parse("Error"))
}
//////
//////
////// 
pub fn file_read_to_json(_filepath: &str) -> serde_json::Result<Value> {
    let pathstring = _filepath;
    println!("{:?}",_filepath);
    match fs::read_to_string(&pathstring) {
        Err(e) => {
            println!("{:?}",e.description());
            Ok(json!(format!("Error:{}",e.description())))
        },
        Ok(file) => serde_json::from_str(&*file),
    }
}

fn filenameload (req: HttpRequest) -> HttpResponse {
    println!("{:?}",req.headers());
    println!("{:?}",req.query_string());
    println!("{:?}",req.match_info().get("param").unwrap());
    HttpResponse::Ok().json(file_read_to_json(&format!("item/{}.json",req.match_info().get("param").unwrap())).unwrap())
}

pub fn save_file(field: Field) -> impl Future<Item = i64, Error = ActixError> {
    let file_path_string = match field.content_disposition().unwrap().get_filename() {
        Some(filename) => filename.replace(' ', "_").to_string(),
        None => return Either::A(err(error::ErrorInternalServerError("Couldn't read the filename.")))
    };
    let filename = match &file_path_string.rfind('/') {
        Some(idx) => {
            let path = format!("static/images/{}", &file_path_string[0..*idx]);
            fs::create_dir_all(&path).expect("make path Fail");
            &file_path_string
        },
        None => &file_path_string
    };
    let file = match fs::File::create(format!("static/images/{}",filename)) {
        Ok(file) => file,
        Err(e) => return Either::A(err(error::ErrorInternalServerError(e))),
    };
    Either::B(
        field
            .fold((file, 0i64), move |(mut file, mut acc), bytes| {
                // fs operations are blocking, we have to execute writes
                // on threadpool
                web::block(move || {
                    file.write_all(bytes.as_ref()).map_err(|e| {
                        println!("file.write_all failed: {:?}", e);
                        MultipartError::Payload(error::PayloadError::Io(e))
                    })?;
                    acc += bytes.len() as i64;
                    Ok((file, acc))
                })
                .map_err(|e: error::BlockingError<MultipartError>| {
                    match e {
                        error::BlockingError::Error(e) => e,
                        error::BlockingError::Canceled => MultipartError::Incomplete,
                    }
                })
            })
            .map(|(_, acc)| acc)
            .map_err(|e| {
                println!("save_file failed, {:?}", e);
                error::ErrorInternalServerError(e)
            }),
    )
}

pub fn uploadimage(
    multipart: Multipart,
    counter: web::Data<Cell<usize>>,
) -> impl Future<Item = HttpResponse, Error = ActixError> {
    counter.set(counter.get() + 1);
    println!("{:?}", counter.get());

    multipart
        .map_err(error::ErrorInternalServerError)
        .map(|field| save_file(field).into_stream())
        .flatten()
        .collect()
        .map(|sizes| HttpResponse::Ok().json(sizes))
        .map_err(|e| {
            println!("failed: {}", e);
            e
        })
}

pub fn uploadjson (req: HttpRequest, _data: web::Json<ViewContent>) -> HttpResponse  {
    let item_no = req.match_info().get("item_no").unwrap();
    let seq = req.match_info().get("seq").unwrap();
    let filename = req.match_info().get("filename").unwrap();

    let path = format!("static/images/{}/{}", &item_no, &seq);
    fs::create_dir_all(&path).expect("make path Fail");
    
    let pathstr = format!("static/images/{}/{}/{}.json", &item_no, &seq, &filename);
    let path = Path::new(&pathstr);
    let mut data: ViewContent = _data.into_inner();
    //data.images = data.images.into_iter().map(|x|format!("{}static/images/{}",load_app_path(),x)).collect();
    let json = serde_json::to_string(&serde_json::to_value(data).unwrap()).unwrap();

    match File::create(&path) {
        Err(e) => {
            println!("failed: {}", e);
            res_error()
        }
        Ok(mut file) => match file.write_all(&json.as_bytes()) {
            Err(e) => {
                println!("failed: {}", e);
                res_error()
            }
            Ok(_d) => actix_web::HttpResponse::Ok().json(json!({"result":"Ok"})),
        },
    }
}

fn index() -> HttpResponse {
    match fs::read_to_string("static/index.html") {
        Err(e) => {
            println!("{:?}",e.description());
            HttpResponse::Ok().body("")
        },
        Ok(html) => HttpResponse::Ok().body(html),
    }
}


fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .data(Cell::new(0usize))
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/uploadimage")
                    .route(web::get().to(index))
                    .route(web::post().to_async(uploadimage)),
            )
            .service(web::resource("/uploadjson/{item_no}/{seq}/{filename}").route(web::post().to_async(uploadjson)))
            .service(web::resource("/static/images").route(web::get().to(filenameload)))
            .service(actfs::Files::new("/static", "static").show_files_listing())
    })
    .bind(load_ip_port())?
    .run()
}
