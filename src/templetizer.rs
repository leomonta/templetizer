#![allow(nonstandard_style)]
#![allow(dead_code)]

use std::env;
use std::fs;
use std::io::Write;

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

fn complete_template(file_data: &str, template_names: &Vec<String>, target_types: &Vec<String>, mut output_file: &fs::File) -> usize {

	let mut old_c_stop: usize = 0;
	let mut c_stop: usize = 0;
	let mut chunk;

	// loop all the {} to detectd the end of the function to templetize
	// Do i need to separate in chunks the function body to remplace the templates? No
	// but I need to deect the end of the function body (with the closing }) so i do both at the same time
	// Is this a good implementation? no, it detects braces in strnigs as normal braces, but oh well
	loop {
		let open_br = file_data.find("{");
		let clos_br = file_data.find("}");

		// close does not exists, open does
		if clos_br.is_none() && open_br.is_some() {
			stack.push('{');
			c_stop = open_br.unwrap();

		// the opposite
		} else if open_br.is_none() && clos_br.is_some() {
			if stack.last().is_some() {
				abort("Wring syntax, '}' without a matching '{'", Void);
			}
			c_stop = clos_br.unwrap();

		// both None
		} else if open_br.is_none() && clos_br.is_none() {
			// we're done here,
			break;

		// the close bracket is first
		} else if Some(open_br) > Some(clos_br) {
			if stack.last().is_some() {
				abort("Wring syntax, '}' without a matching '{'", Void);
			}
			c_stop = clos_br.unwrap();

		// the open bracket is first
		} else if Some(open_br) < Some(clos_br) {
			stack.push('{');
			c_stop = open_br.unwrap();
		}

		chunk = &file_data[old_c_stop..c_stop];

		// replace all of the template types in the chunk
		loop {
			let mut found = false;
			for i in 0..template_names.len() {
				match file_data.find(&template_names[i]) {
					| Some(val) => {
						output_file.write(file_data[old_c_stop..val].as_bytes()).expect("Failed Write");
						output_file.write(target_types[i].as_bytes()).expect("Failed Write");
						old_c_stop = val + template_names[i].len();
						found = true;
					}
					| None => (),
				};
			}
			if !found {
				break;
			}
		}

		let line = &file_data[old_c_stop..c_stop];
		old_c_stop = c_stop;

		output_file.write(line.as_bytes()).expect("Failed Write");
		break;
	}

	return nl;
}

/// Given the template declaration (`template <T, U, V, ...>`)
/// returns the template types (`T`, `U`, `V`) as a `Vec` of owned `String`s
fn parse_templated_names(file_data: &str) -> (Vec<String>, usize) {
	let mut res: Vec<String> = Vec::new();

	let open_br = match file_data.find("<") {
		| Some(val) => val + 1,
		| None => abort("Invalid template syntax, missing '<'", Void),
	};

	let clos_br = match file_data.find(">") {
		| Some(val) => val,
		| None => abort("Invalid template syntax, missing '>'", Void),
	};

	let slice = &file_data[open_br..clos_br];

	let parts = slice.split(",");

	for p in parts {
		res.push(p.trim().to_owned());
	}

	return (res, clos_br + 1);
}

fn main() {
	let args: Vec<String> = env::args().collect();

	match args.len() {
		| 1 => abort("Not enough arguments: the first argument must be the target template", Void),
		| 2 => abort("Not enough arguments: the second argument must be the type to complete the template with", Void),
		| _ => (),
	}

	let target_filename: &str = &args[1];
	let target_types = &args[2..].to_vec();

	let file_data = match fs::read_to_string(target_filename) {
		| Ok(dt) => dt,
		| Err(e) => abort(&format!("Could not read from the file{target_filename}"), Some(e)),
	};

	let mut templated_names;
	let mut old_nl: usize = 0;
	let mut nl: usize = 0;
	let mut output_file = fs::File::create("tl.out").expect("Failed Create");

	// reading line by line
	loop {
		match file_data.find("\n") {
			| Some(val) => nl = val,
			| None => break,
		};

		let line = &file_data[old_nl..nl];

		if line.contains(TEMPLATE_KEY_WORD_START) {
			(templated_names, old_nl) = parse_templated_names(&file_data[old_nl..]);
			dbg!(&templated_names);

			old_nl = complete_template(&file_data[old_nl..], &templated_names, target_types, &output_file);
		} else {
			output_file.write(line.as_bytes()).expect("Failed Write");
			old_nl = nl;
		}
	}
}
