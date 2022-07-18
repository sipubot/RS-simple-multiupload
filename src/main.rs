#[macro_use]
extern crate serde_derive;

use std::cell::Cell;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use actix_multipart::Multipart;
use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use futures_util::TryStreamExt as _;

use serde_json::{ json, Value};

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
#[allow(dead_code)]
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
            println!("{:?}",e);
            Ok(json!(format!("Error:{}",e)))
        },
        Ok(file) => serde_json::from_str(&*file),
    }
}

async fn filenameload (req: HttpRequest) -> HttpResponse {
    println!("{:?}",req.headers());
    println!("{:?}",req.query_string());
    println!("{:?}",req.match_info().get("param").unwrap());
    HttpResponse::Ok().json(file_read_to_json(&format!("item/{}.json",req.match_info().get("param").unwrap())).unwrap())
}

pub async fn uploadjson (req: HttpRequest, _data: web::Json<ViewContent>) -> HttpResponse  {
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

async fn index() -> HttpResponse {
    match fs::read_to_string("static/index.html") {
        Err(e) => {
            println!("{:?}",e);
            HttpResponse::Ok().body("")
        },
        Ok(html) => HttpResponse::Ok().body(html),
    }
}

async fn uploadimage(mut multipart: Multipart, counter: web::Data<Cell<usize>>,) -> actix_web::Result<HttpResponse, actix_web::Error> {
    counter.set(counter.get() + 1);
    println!("{:?}", counter.get());

    // iterate over multipart stream
    while let Some(mut field) = multipart.try_next().await? {
        // A multipart/form-data stream has to contain `content_disposition`
        let content_disposition = field.content_disposition();

        let mut file_path_string = content_disposition
            .get_filename()
            .map_or_else(|| uuid::Uuid::new_v4().to_string(), sanitize);

        file_path_string = file_path_string.replace(' ', "_").to_string();
        let filename = match &file_path_string.rfind('/') {
            Some(idx) => {
                let path = format!("static/images/{}", &file_path_string[0..*idx]);
                fs::create_dir_all(&path).expect("make path Fail");
                &file_path_string
            },
            None => &file_path_string
        };

        let filepath = format!("static/images/{filename}");
        // File::create is blocking operation, use threadpool
        let mut f = web::block(|| std::fs::File::create(filepath)).await??;

        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.try_next().await? {
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
        }
    }
    Ok(HttpResponse::Ok().into())
}

pub fn sanitize<S: AsRef<str>>(name: S) -> String {
    "flisjhfskjrk3wshkwsef".to_string()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    std::fs::create_dir_all("./tmp")?;

    HttpServer::new(|| {
        App::new().wrap(middleware::Logger::default()).service(
            web::resource("/uploadimage")
                .route(web::get().to(index))
                .route(web::post().to(uploadimage)),
        )
        .service(web::resource("/uploadjson/{item_no}/{seq}/{filename}").route(web::post().to(uploadjson)))
        .service(web::resource("/static/images").route(web::get().to(filenameload)))
    })
    .bind(load_ip_port())?
    .run()
    .await
}
