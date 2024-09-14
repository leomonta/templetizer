#![allow(nonstandard_style)]
#![allow(dead_code)]

use std::time;
use std::env;
use std::fs;
use std::io::Write;
use std::thread;
use std::usize;

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


fn get_func_end(mut file_data: &str) -> usize {

	let mut stack: Vec<char> = Vec::new();
	let mut res: usize = 0;
	let mut stop: usize = 0;

	loop {
		let open_br = file_data.find("{").unwrap_or(usize::MAX);
		let clos_br = file_data.find("}").unwrap_or(usize::MAX);

		// both paren were not found,
		// we're done here
		if open_br == clos_br {
			break;

		// the close bracket is first
		} else if open_br > clos_br {
			if !stack.last().is_some() {
				return abort("Wring syntax, '}' without a matching '{'", Void);
			}
			stack.pop().unwrap();
			stop = clos_br + 1;

		// the open bracket is first
		} else if open_br < clos_br {
			stack.push('{');
			stop = open_br + 1;
		}

		res += stop;

		if stack.is_empty() {
			break;
		}

		file_data = &file_data[stop..];
	}

	return res;
}

fn complete_template(file_data: &str, template_names: &Vec<String>, target_types: &Vec<String>, mut output_file: &fs::File) -> usize {
	
	return 0;

	/*
	This function, to work nicely, needs a lexer, a tokenizer, a CFG decoder, however the fuck is called the thing in the compiler that recognizes keywords, operators, and names.
	But I ain't gonna do that. 
	Not even gonna fucking try. C isn't super diffucult (except things like function pointers typedefs) but still.. No

	I'm gonna assume that no one is feeding this tool minified C code (if such a thing even exists), so i will assume that all template types have at least a space after them
	(thing I'm pretty sure is obligatory) and check for opening and closign brackets, '{' and '}',
	
	This means that anything inside comments will not be treated as such, so you might fuck up the function end detection with brackets inside comments, and the templated type will be replaced  inside of them

	good lucl
	*/



	/*
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
	*/
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

			let func_end = old_nl + get_func_end(&file_data[old_nl..]);
			println!("{}", &file_data[old_nl..func_end]);
			return;
			old_nl = complete_template(&file_data[old_nl..func_end], &templated_names, target_types, &output_file);
		} else {
			output_file.write(line.as_bytes()).expect("Failed Write");
			old_nl = nl;
		}
	}
}
