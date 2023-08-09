use clap::{Parser, Subcommand};
use inquire::{validator::Validation, CustomType};
use owo_colors::OwoColorize;
use tabled::builder::Builder;
use tabled::settings::{
    object::{Columns, Rows},
    peaker::PriorityMax,
    style::BorderColor,
    Alignment, Color, Format, Modify, Panel, Style, Width,
};
use terminal_size::{terminal_size, Width as TermWidth};

#[derive(Debug, Subcommand)]
pub enum Commands {
    Add { release: String },
    Cart,
    Wantlist,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    pub cookies: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug)]
pub enum TableType {
    Default,
    Cart,
}

pub fn print_table(
    header: &'static [&str],
    table: &Vec<Vec<String>>,
    title: &str,
    table_type: TableType,
) {
    let mut builder = Builder::default();

    builder.set_header(header.to_vec());
    for row in table.iter() {
        builder.push_record(row);
    }

    let mut formatted_table = match table_type {
        TableType::Default => builder.index().build(),
        TableType::Cart => builder.build(),
    };

    let (TermWidth(term_width), _) = terminal_size().expect("Failed to get terminal size.");

    formatted_table
        .with(Panel::header(title))
        .with(Modify::new(Rows::first()).with(Alignment::center()))
        .with(Style::rounded().horizontal('-'))
        .with(BorderColor::filled(Color::FG_BLUE))
        .with(Width::wrap(term_width as usize).priority::<PriorityMax>());

    match table_type {
        TableType::Default => formatted_table
            .with(Modify::new(Columns::first()).with(Format::content(|s| s.red().to_string()))),
        TableType::Cart => {
            formatted_table
                .with(Modify::new(Rows::first()).with(Format::content(|s| s.red().to_string())));
            // +2 Because of title and header
            let subtotal_index = table.iter().position(|row| row[0] == "Subtotal").unwrap() + 2;
            formatted_table.with(
                Modify::new(Rows::new(subtotal_index..))
                    .with(Format::content(|s| s.red().to_string())),
            )
        }
    };

    println!("{}", formatted_table);
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
