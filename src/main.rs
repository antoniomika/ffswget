#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate lazy_static;
extern crate ffsend_api;
extern crate mime_guess;

use ffsend_api::pipe::crypto::EceCrypt;
use ffsend_api::pipe::prelude::*;

lazy_static! {
    static ref HOST: String = get_host();
    static ref MAIN_URL: String = get_url();
}

fn get_host() -> String {
    let args: Vec<String> = std::env::args().collect();
    let host: String;

    if args.len() > 1 {
        host = args[1].clone();
    } else {
        host = "http://localhost:8000".to_string();
    }

    return host
}

fn get_url() -> String {
    let args: Vec<String> = std::env::args().collect();
    let main_url: String;

    if args.len() > 2 {
        main_url = args[2].clone();
    } else {
        main_url = "https://send.firefox.com".to_string();
    }

    return main_url
}

struct DownloadStream(rocket::response::Stream<ffsend_api::pipe::crypto::EceReader>, ffsend_api::action::metadata::MetadataResponse);

impl<'r> rocket::response::Responder<'r> for DownloadStream {
    fn respond_to(self, req: &rocket::Request) -> rocket::response::Result<'r> {
        return rocket::Response::build_from(self.0.respond_to(req)?)
            .raw_header("Content-Disposition", format!("attachment; filename=\"{}\"", self.1.metadata().name()))
            .raw_header("Content-Length", self.1.metadata().size().unwrap().to_string())
            .ok();
    }
}

struct ContentLength(u64, String);
impl<'a, 'r> rocket::request::FromRequest<'a, 'r> for ContentLength {
    type Error = ();
    fn from_request(request: &'a rocket::Request<'r>) -> rocket::request::Outcome<ContentLength, ()> {
        rocket::Outcome::Success(ContentLength(request.headers().get_one("Content-Length").unwrap_or_default().parse::<u64>().unwrap_or_default(), request.headers().get_one("Host").unwrap_or_default().to_string()))
    }
}

#[put("/<file>", data = "<data>")]
fn upload_file(file: String, data: rocket::Data, len: ContentLength) -> String {
    handle_upload(file, data, len)
}

#[get("/<download>/<key>")]
fn download_parts(download: String, key: String) -> Result<DownloadStream, rocket::response::status::NotFound<String>> {
    handle_download(format!("{}/download/{}/#{}", *MAIN_URL, download, key))
}

#[get("/download?<url>")]
fn download_url(url: String) -> Result<DownloadStream, rocket::response::status::NotFound<String>> {
    handle_download(url)
}

fn handle_upload(file_name: String, file_data: rocket::Data, len: ContentLength) -> String {
    let client_config = ffsend_api::client::ClientConfig::default();
    let upload_client = client_config.client(true);

    let fake_path = std::path::Path::new(file_name.as_str());
    let file = ffsend_api::action::upload::FileData{
        name: file_name.as_str(),
        mime: mime_guess::from_path(fake_path.clone()).first_or_octet_stream(),
        size: len.0
    };

    let key = ffsend_api::crypto::key_set::KeySet::generate(true);
    let ikm = key.secret().to_vec();
    let encrypt = EceCrypt::encrypt(len.0 as usize, ikm, None);
    let reader = encrypt.reader(Box::new(file_data.open()));

    let url_data = ffsend_api::url::Url::parse(&MAIN_URL).unwrap();

    let upload_call = ffsend_api::action::upload::Upload::new(
        ffsend_api::api::Version::V3,
        url_data,
        std::path::PathBuf::from(fake_path.clone()),
        Some(file_name.clone()),
        None,
        None
    );

    match upload_call.upload_send3(&upload_client, &key, &file, ffsend_api::action::upload::Reader::new(Box::new(reader))) {
        Ok(i) => {
            let id = i.0.id;
            let secret = ffsend_api::crypto::b64::encode(i.0.secret.as_slice());
            format!("{}/{}/{}\n{}/download/{}/#{}\n", *HOST, id, secret.to_string(), *MAIN_URL, id, secret.to_string())
        },
        Err(e) => format!("{:?}", e)
    }
}

fn handle_download(download_url: String) -> Result<DownloadStream, rocket::response::status::NotFound<String>> {
    let client_config = ffsend_api::client::ClientConfig::default();
    let download_client = client_config.client(true);

    let data_url = match ffsend_api::url::Url::parse(&download_url) {
        Ok(i) => i,
        Err(e) => {
            println!("{}", e);
            return Err(rocket::response::status::NotFound(format!("Unable to parse data url {:?}", e)))
        },
    };

    let file = match ffsend_api::file::remote_file::RemoteFile::parse_url(data_url, None) {
        Ok(i) => i,
        Err(e) => {
            println!("{}", e);
            return Err(rocket::response::status::NotFound(format!("Unable to parse file {:?}", e)))
        }
    };

    let meta_res = match ffsend_api::action::metadata::Metadata::new(&file, None, true).invoke(&download_client) {
        Ok(i) => i,
        Err(e) => {
            println!("meta {}", e);
            return Err(rocket::response::status::NotFound(format!("Unable to retrieve metadata {:?}", e)))
        }
    };

    let path_buf = std::path::PathBuf::from(format!("/tmp/{}", meta_res.metadata().name()));

    let download_call = ffsend_api::action::download::Download::new(
        ffsend_api::api::Version::V3,
        &file,
        path_buf.clone(),
        None,
        false,
        Some(meta_res.clone())
    );

    let key = ffsend_api::crypto::key_set::KeySet::from(
        &file,
        None
    );

    let ikm = key.secret().to_vec();

    let (reader, len) = match download_call.create_file_reader(&key, &meta_res, &download_client) {
        Ok(i) => i,
        Err(e) => return Err(rocket::response::status::NotFound(format!("Not found {:?}", e)))
    };

    let decrypt_reader = EceCrypt::decrypt(len as usize, ikm.clone()).reader(Box::new(reader));

    Ok(DownloadStream(rocket::response::Stream::from(decrypt_reader), meta_res.clone()))
}

fn main() {
    rocket::ignite().mount("/", routes![upload_file, download_parts, download_url]).launch();
}
