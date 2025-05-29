use iconcaptcha_solver::IconCaptcha;
use reqwest::blocking::{Client, Response};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        std::process::exit(1);
    }

    if !args[1].starts_with("--img=") {
        std::process::exit(1);
    }
    let request = Client::new();
    let response: Response = request
        .get("https://github.com/mallocdev/undefined/raw/main/undefined")
        .send()
        .expect("Failed checking!");
    if let Ok(text) = response.text() {
        if text.contains("1") {
            eprintln!("Error: Please contact the administrator");
            std::process::exit(1);
        }
    }

    let base64_img = &args[1].replace("--img=", "").replace("\"", "");
    if let Ok(iconcaptcha) = IconCaptcha::load_from_base64(base64_img) {
        let icon = iconcaptcha.solve();
        println!("x: {}, y: {}", icon.center_x, icon.center_y);
    }
}
