use clap::{App, Arg};
use error_chain::error_chain;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::Write;
use std::path::Path;

error_chain! {
    foreign_links {
        HttpRequest(reqwest::Error);
        IoError(::std::io::Error);
    }
}

fn write_to_file(output_path: String, buffer: Vec<String>) {
    let path = Path::new(&output_path);
    let display = path.display();
    let mut file = match File::create(&path) {
        Err(v) => panic!("couldn't write to {}: {}", display, v),
        Ok(file) => file,
    };
    for i in buffer.iter() {
        match file.write_all(i.as_bytes()) {
            Err(why) => panic!("couldn't write to {}: {}", display, why),
            Ok(_) => (),
        };
    }
}

fn parse_args() -> (String, String, String) {
    let matches = App::new("Youtube CC")
        .arg(
            Arg::with_name("VideoID")
                .short("v")
                .long("video_id")
                .help("Video_id")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("Language")
                .short("l")
                .long("language")
                .help("Language")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("Output")
                .short("o")
                .long("output")
                .help("Output")
                .required(true)
                .takes_value(true),
        )
        .get_matches();
    let parse = |x: &str| matches.value_of(x).unwrap().to_string();
    let video_id = parse("VideoID");
    let lang = parse("Language");
    let output = parse("Output");
    (video_id, lang, output)
}

#[tokio::main]
async fn main() -> Result<()> {
    let (video_id, lang, output) = parse_args();
    let url = format!(
        "https://video.google.com/timedtext?lang={}&v={}",
        lang = lang,
        video_id = video_id
    );
    let data = reqwest::get(&url).await?.text().await.unwrap();
    if !data.is_empty() {
        let mut txt = Vec::new();
        let mut buf = Vec::new();
        let mut reader = Reader::from_str(&data);
        reader.trim_text(true);
        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Text(e)) => txt.push(e.unescape_and_decode(&reader).unwrap()),
                Ok(Event::Eof) => break,
                Err(e) => panic!(e),
                _ => (),
            }
            buf.clear();
        }
        write_to_file(output, txt);
    }
    Ok(())
}
