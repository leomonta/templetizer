#![allow(nonstandard_style)]
#![allow(dead_code)]

use std::env;
use std::fs;
use std::io::Write;
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

fn min_index(arr: &Vec<usize>) -> usize {
	// wow, such algorithm
	let mut min = 0;
	for i in 1..arr.len() {
		if arr[i] < arr[min] {
			min = i;
		}
	}

	return min;
}

/// given a slice containing a function, replaces all of the template types with the target types
fn complete_template(mut file_data: &str, template_names: &Vec<String>, target_types: &Vec<String>, mut output_file: &fs::File) {
	/*
	This function, to work nicely, needs a lexer, a tokenizer, a CFG decoder, however the fuck is called the thing in the compiler that recognizes keywords, operators, and names.
	But I ain't gonna do that.
	Not even gonna fucking try. C isn't super diffucult (except things like function pointers typedefs) but still.. No

	I'm gonna assume that no one is feeding this tool minified C code (if such a thing even exists), so i will assume that all template types have at least a space after them
	(thing I'm pretty sure is obligatory) and check for opening and closign brackets, '{' and '}',

	This means that anything inside comments will not be treated as such, so you might fuck up the function end detection with brackets inside comments, and the templated type will be replaced  inside of them

	good luck
	*/

	// positions of all the templates types in the file
	let mut positions: Vec<usize> = vec![0; template_names.len()];

	loop {
		// find all template types
		for i in 0..template_names.len() {
			positions[i] = file_data.find(&template_names[i]).unwrap_or(usize::MAX);
		}

		// and select the closest
		let next = min_index(&positions);

		// no template type has been found
		if positions[next] == usize::MAX {
			break;
		}

		output_file.write(file_data[..positions[next]].as_bytes()).expect("Failed Write");
		output_file.write(target_types[next].as_bytes()).expect("Failed Write");

		file_data = &file_data[positions[next] + 1..];
	}

	// print the rest of the chunk
	output_file.write(file_data.as_bytes()).expect("Failed Write");
}

/// Given the template declaration (`template <T, U, V, ...>`)
/// returns the template placeholders types (`T`, `U`, `V`) as a `Vec` of owned `String`s
fn parse_template_placeholders(file_data: &str) -> (Vec<String>, usize) {
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

	let file = match fs::read_to_string(target_filename) {
		| Ok(dt) => dt,
		| Err(e) => abort(&format!("Could not read from the file {target_filename}"), Some(e)),
	};

	// WTH Rust WTH
	let mut file_data = &file[0..];

	let mut template_placeholders;
	let mut nl: usize = 0;
	let mut old_nl: usize;
	let mut output_file = fs::File::create("tl.out").expect("Failed Create");
	let mut line_num: usize = 0;

	// reading line by line
	loop {
		old_nl = nl;
		match file_data.find("\n") {
			| Some(val) => nl = val,
			| None => break,
		};

		let line = &file_data[old_nl..nl];

		line_num += 1;

		if !line.contains(TEMPLATE_KEY_WORD_START) {
			output_file.write(line.as_bytes()).expect("Failed Write");
			continue;
		} else {
			(template_placeholders, old_nl) = parse_template_placeholders(&file_data[old_nl..]);

			let ph_len = template_placeholders.len();
			let tt_len = target_types.len();

			if tt_len != ph_len {
				abort::<i32, Dummy>(&format!("The target types ({tt_len}) do not match the number of template placeholders ({ph_len}) at line {line_num}"), Void);
			}

			// precalculate the boundaries of the function to simplify the template completion
			// Yes, this is double work, it can be improved. I'll do it when i'll run into performance problem
			let func_end = old_nl + get_func_end(&file_data[old_nl..]);
			complete_template(&file_data[old_nl..func_end], &template_placeholders, target_types, &output_file);
			nl = func_end;
		}

		file_data = &file_data[nl+1..];
	}
}
