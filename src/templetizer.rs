#![allow(nonstandard_style)]
#![allow(dead_code)]

use std::env;
use std::fs;

// constants
const INTERNAL_WILDCARD: char = '*';
const TEMPLATE_KEY_WORD_START: &str = "template";
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

fn parse_templated_tyoes(s: &str) -> Vec<String> {
	let mut res: Vec<String> = Vec::new();

	let open_br = match s.find("<") {
		| Some(val) => val + 1,
		| None => abort("Invalid template syntax, missing '<'", Void),
	};

	let clos_br = match s.find(">") {
		| Some(val) => val,
		| None => abort("Invalid template syntax, missing '>'", Void),
	};

	let slice = &s[open_br..clos_br];

	let parts = slice.split(",");

	for p in parts {
		res.push(p.trim().to_owned());
	}

	return res;
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

	let file_data = match fs::read_to_string(target_filename) {
		| Ok(dt) => dt,
		| Err(e) => abort(&format!("Could not read from the file{target_filename}"), Some(e)),
	};

	let mut templated_names;

	let mut old_nl = 0;

	// reading line by line
	loop {
		let nl = match file_data.find("\n") {
			| Some(val) => val + 1,
			| None => abort("Invalid template syntax, missing '<'", Void),
		};

		let line = &file_data[old_nl..nl];
		old_nl = nl;

		if line.contains(TEMPLATE_KEY_WORD_START) {
			templated_names = parse_templated_tyoes(line);
			dbg!(templated_names);

			// let end_templ = get_end_next_bracket()
		}
	}
}
