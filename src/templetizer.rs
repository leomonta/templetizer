use std::env;
use std::fs;
use std::io;
use std::collections;
use std::io::BufRead; // i should not need this

// constants
const INTERNAL_WILDCARD: char = '*';
const TEMPLATE_KEY_WORD_START: &str = "template<";
const TEMPLATE_KEY_WORD_END: &str = ">";

// God forsaken code here
struct Dummy {}

impl std::fmt::Debug for Dummy {
	fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
		return Ok(());
	}
}

const Void: Option<Dummy> = Option::<Dummy>::None;

fn abort<T, U: std::fmt::Debug>(s: &str, err: Option<U>) -> T {
	match err {
		| Some(e) => eprintln!("Aborting: {}\n\t{:#?}", s, e),
		| None => eprintln!("Aborting: {}", s),
	}
	std::process::exit(1);
}

fn parse_templated_tyoes(s: &String) -> collections::HashMap<String, String> {
	
}

fn main() {
	let args: Vec<String> = env::args().collect();

	match args.len() {
		| 1 => abort("Not enough arguments: the first argument must be the target template", Void),
		| 2 => abort("Not enough arguments: the second argument must be the type to complete the template with", Void),
		| _ => (),
	}

	let target_filename: &str = &args[1];
	let target_type: &str = &args[2];

	let target_file = match fs::OpenOptions::new().append(true).read(true).open(target_filename) {
		| Ok(f) => f,
		| Err(e) => abort("Failed to open the file", Some(e)),
	};

	// cuz reading is too hard without a reader
	let mut reader = io::BufReader::new(target_file);

	let mut templated_types;

	// reading line by line
	loop {
		let mut line: String = String::new();
		let read_result = reader.read_line(&mut line);
		match read_result {
			| Ok(0) => break,
			| Err(e) => abort(&format!("Could not read from the file{target_filename}"), Some(e)),
			| _ => (),
		}
		print!("{}", line);

		if line.contains(TEMPLATE_KEY_WORD_START) {
			templated_types = parse_templated_tyoes(&line);
		}
	}
}
