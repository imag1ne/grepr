mod app;
mod error;
mod matcher;

#[macro_use]
extern crate clap;

use crate::app::GreprApp;

fn main() {
    let app = GreprApp::new();

    match app.grep() {
        Ok(_) => (),
        Err(e) => eprintln!("{:?}", e),
    }
}
