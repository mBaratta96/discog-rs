use clap::Parser;
use inquire::{validator::Validation, CustomType};
use owo_colors::OwoColorize;
use tabled::builder::Builder;
use tabled::settings::Format;
use tabled::settings::{
    object::Columns, peaker::PriorityMax, style::BorderColor, Color, Modify, Style, Width,
};
use terminal_size::{terminal_size, Width as TermWidth};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    pub cookies: String,

    #[arg(short, long, default_value_t = String::from(""))]
    pub wantlist: String,
}

pub fn print_table(header: Vec<&str>, table: &Vec<Vec<String>>) {
    let mut builder = Builder::default();

    builder.set_header(header);
    for row in table.iter() {
        builder.push_record(row);
    }

    let builder = builder.index();
    let mut table = builder.build();
    let (TermWidth(term_width), _) = terminal_size().expect("Failed to get terminal size.");

    table
        .with(Style::rounded().horizontal('-'))
        .with(BorderColor::filled(Color::FG_BLUE))
        .with(Modify::new(Columns::single(0)).with(Format::content(|s| s.red().to_string())))
        .with(Width::wrap(term_width as usize).priority::<PriorityMax>());

    println!("{}", table);
}

pub fn ask_id(len: i32, request: &str) -> i32 {
    let selection = CustomType::<i32>::new(request)
        .with_validator(move |input: &i32| {
            if (-1..=len).contains(input) {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid("Input outside of index range.".into()))
            }
        })
        .prompt();
    selection.unwrap()
}
