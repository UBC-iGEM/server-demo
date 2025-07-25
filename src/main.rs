use std::{env, error::Error, fmt::Display, fs::read, io::Write, net::TcpListener, thread};
use tungstenite::accept;

enum ContentType<'a> {
    Html,
    Css,
    Js,
    Image(&'a str),
}
impl<'a> Display for ContentType<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const BASE: &str = "Content-Type:";
        let ty = match self {
            Self::Html => "text/html".to_string(),
            Self::Css => "text/css".to_string(),
            Self::Js => "application/javascript".to_string(),
            Self::Image(ty) => format!("image/{ty}"),
        };
        write!(f, "{BASE} {ty}")
    }
}

static GENERATE_RESPONSE: fn(ContentType, Vec<u8>) -> Vec<u8> =
    |ty: ContentType, content: Vec<u8>| -> Vec<u8> {
        let len = content.len();
        let mut response =
            format!("HTTP/1.1 200 OK\r\nContent-Length: {len}\r\nContent-Type: {ty}\r\n\r\n")
                .into_bytes();
        response.extend_from_slice(&content);
        response
    };

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:9999")?;
    let exe_origin = env::current_exe()?;
    let bundle_dir = exe_origin
        .parent()
        .ok_or(Box::<dyn Error>::from("Bundle directory is malfored!"))?
        .join("public/");

    for connection in listener.incoming() {
        let mut conn = connection?;
        let mut buffer = [0; 1024];
        // Use peek over read, lest we consume part of the stream
        let len = conn.peek(&mut buffer)?;
        if len == 0 {
            continue;
        }
        let request = String::from_utf8_lossy(&buffer[0..len]);
        if let Some(line) = request.lines().next() {
            let mut components = line.split_whitespace();
            // Components should be METHOD, PATH, PROTOCOL
            if let Some(mut path) = components.nth(1) {
                if path == "/" {
                    path = "/index.html";
                }
                if path == "/ws" {
                    let conn_clone = conn.try_clone()?;
                    thread::spawn(|| {
                        let mut ws = accept(conn_clone).unwrap();
                        loop {
                            let msg = ws.read().unwrap();
                            println!("WS message read: {msg}");
                            if msg.is_text() {
                                ws.send(msg).unwrap();
                            }
                        }
                    });
                    continue;
                }
                // Splice out leading backslash
                let bundle_asset = bundle_dir.join(&path[1..]);
                println!("Requested asset: {bundle_asset:?}");
                if bundle_asset.exists() {
                    let asset_body = read(&bundle_asset)?;
                    let extension = bundle_asset
                        .extension()
                        .ok_or(Box::<dyn Error>::from(
                            "Asset without extension was requested!",
                        ))?
                        .to_string_lossy();
                    let content_type = match extension.as_ref() {
                        "html" => ContentType::Html,
                        "css" => ContentType::Css,
                        "js" => ContentType::Js,
                        _ => ContentType::Image(&extension),
                    };
                    let response = GENERATE_RESPONSE(content_type, asset_body);
                    conn.write_all(&response)?;
                };
            }
        }
    }
    Ok(())
}
