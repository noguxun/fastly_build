use anyhow::{anyhow, Result};
use crates_io_api::AsyncClient;
use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Method, Request, Response, Server, StatusCode};
use lazy_static::lazy_static;
use rand::{distributions::Alphanumeric, rngs::SmallRng, Rng, SeedableRng};
use sled::Db;
use std::{env, fs::OpenOptions, io::prelude::*, process::Command};

lazy_static! {
    static ref BUILD_DB: Db = sled::open("./build_result_db").unwrap();
}

fn get_build_result(crate_dep: &str) -> Result<u8> {
    let result = BUILD_DB.get(crate_dep)?;
    if result.is_none() {
        return Err(anyhow!("Not found"));
    }

    let value = result.unwrap();
    Ok(value[0])
}

fn save_build_result(crate_dep: &str, result: u8) {
    BUILD_DB.insert(crate_dep, &[result]).unwrap();
}

async fn hyper_service(req: Request<Body>) -> Result<Response<Body>> {
    if req.method() != Method::GET {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("foo", "bar")
            .body(Body::empty())?);
    }

    let path = req.uri().path();
    println!("GET {}", path);

    let crate_name = urlencoding::decode(&path[1..])?;
    let version_result = get_crate_version(&crate_name).await;
    if version_result.is_err() {
        let body_str = format!(
            include!("result_template.html"),
            crate_name, "NO VERSION", "FAILED"
        );
        let resp = Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(body_str))?;
        return Ok(resp);
    }

    let version = version_result.unwrap();
    let crate_dep = format!("{} = \"{}\"", crate_name, version);

    println!("{}", crate_dep);

    if let Ok(result) = get_build_result(&crate_dep) {
        let (build_resut, age) = match result {
            0 => ("FAILED", 36000000),
            1 => ("OK", 36000000),
            _ => ("Building ... refresh the page after a minute", 5),
        };

        let body_str = format!(
            include!("result_template.html"),
            crate_name, version, build_resut
        );

        let resp = Response::builder()
            .status(StatusCode::OK)
            .header(header::CACHE_CONTROL, format!("max-age={}", age))
            .body(Body::from(body_str))?;

        Ok(resp)
    } else {
        // label it is still being build
        // TODO: this is not still solve the simutaneously request problem
        save_build_result(&crate_dep, 2);

        tokio::task::spawn(build_crate_wasm32(crate_dep));

        let body_str = format!(
            include!("result_template.html"),
            crate_name, version, "Building ... refresh the page after a minute"
        );

        let resp = Response::builder()
            .status(StatusCode::OK)
            .header(header::CACHE_CONTROL, "max-age=5")
            .body(Body::from(body_str))?;

        Ok(resp)
    }
}

async fn build_crate_wasm32(crate_dep: String) -> Result<()> {
    println!("starting to build {}", crate_dep);
    // create new project folder name
    let rand_string: String = SmallRng::from_entropy()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();

    let new_build_folder = format!("build-test-{}", rand_string);

    // copy template project to a new folder
    let mut output = Command::new("cp")
        .arg("-a")
        .arg("build-test")
        .arg(&new_build_folder)
        .output()?;

    if !output.status.success() {
        let err_msg = format!(
            "{} {}",
            String::from_utf8(output.stdout)?,
            String::from_utf8(output.stderr)?
        );
        return Err(anyhow!(err_msg));
    }

    // cd into new folder
    env::set_current_dir(&new_build_folder)?;

    // append the crate dependency to cargo.toml
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("Cargo.toml")
        .unwrap();

    if let Err(e) = writeln!(file, "{}", crate_dep) {
        eprintln!("Couldn't write to file: {}", e);
    }

    // kick off the build
    output = Command::new("cargo").arg("check").output()?;
    let build_result = if output.status.success() { 1 } else { 0 };

    save_build_result(&crate_dep, build_result);

    println!(
        "{} \n {} \n {} \n",
        output.status.success(),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap()
    );

    // get out of new directory
    env::set_current_dir("..")?;

    // remove the new directory
    Command::new("rm")
        .arg("-rf")
        .arg(&new_build_folder)
        .output()?;

    Ok(())
}

async fn get_crate_version(crate_name: &str) -> Result<String> {
    // Instantiate the client.
    let client = AsyncClient::new(
        "build-wasm32-check (yesguxun@gmail.com)",
        std::time::Duration::from_millis(1000),
    )?;

    let crate_resp = client.get_crate(crate_name).await?;

    return Ok(crate_resp.crate_data.max_version);
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<()> {
    let addr = ([0, 0, 0, 0], 8080).into();
    let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(hyper_service)) });
    println!("Starting to serve on http://{}", addr);

    let server = Server::bind(&addr).serve(service);
    if let Err(e) = server.await {
        println!("server error: {}", e);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn build_crate_wasm32_works() {
        assert_eq!(
            build_crate_wasm32(r#"rust-crypto = "^0.3""#.to_string()).unwrap(),
            false
        );
        assert_eq!(
            build_crate_wasm32(r#"rust-crypto-wasm = "^0.2""#.to_string()).unwrap(),
            true
        );

        assert_eq!(
            build_crate_wasm32(r#"fastly="^0.6""#.to_string()).unwrap(),
            true
        );
    }

    #[test]
    fn db_work() {
        // insert and get, similar to std's BTreeMap
        BUILD_DB.insert("key", "value").unwrap();
        assert_eq!(
            BUILD_DB.get(&"key").unwrap(),
            Some(sled::IVec::from("value")),
        );

        BUILD_DB.insert("fastly=\"0.7.1\"", &[0u8]).unwrap();
        assert_eq!(
            BUILD_DB.get(&"fastly=\"0.7.1\"").unwrap(),
            Some(sled::IVec::from(&[0u8])),
        );

        BUILD_DB.insert("fastly=\"0.7.1\"", &[1u8]).unwrap();
        assert_eq!(
            BUILD_DB.get(&"fastly=\"0.7.1\"").unwrap(),
            Some(sled::IVec::from(&[1u8])),
        );

        save_build_result("fastly=\"0.7.2\"", 1);

        let result = get_build_result("fastly=\"0.7.2\"");
        assert_eq!(result.unwrap(), 1u8);
    }

    #[tokio::test]
    async fn get_crate_version_work() {
        assert_eq!(get_crate_version("fastly").await.unwrap(), "0.7.1");
    }
}
