use inquire::{validator::Validation, CustomType};
use tabled::builder::Builder;
use tabled::settings::{peaker::PriorityMax, style::BorderColor, Color, Style, Width};
use terminal_size::{terminal_size, Width as TermWidth};

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
