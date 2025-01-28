#![allow(nonstandard_style)]

/*
Glossary, to clarify a couple of concepts
template type: S, T, U, V, ... when considered as actual type, means kinda the same as template name
template name: S, T, U, V, ... when considered as text

*/

use std::env;
use std::fs;
use std::io::Write;
use std::process::exit;
use std::usize;

extern crate regex;
use regex::Match;
use regex::Regex;

// constants
const INTERNAL_WILDCARD: char = '*';
const TEMPLATE_DECLARATION_KEYWORD: &str = "template";
const TEMPLATE_KEY_WORD_END: &str = ">";

// God forsaken code here
struct Dummy {}

impl std::fmt::Debug for Dummy {
	fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
		return Ok(());
	}
}

const Void: Option<Dummy> = Option::<Dummy>::None;

/// The whole point of this function is to exit the program with an error printed, I later found out about `.expect(...)`
/// but this is still usefull in some occasions
fn abort<T, U: std::fmt::Debug>(s: &str, err: Option<U>) -> T {
	match err {
		| Some(e) => eprintln!("Aborting: {}\n\t{:#?}", s, e),
		| None => eprintln!("Aborting: {}", s),
	}
	std::process::exit(1);
}

/// returns the position, for the given string ref, of the first `\n` after the closing curly bracket `}` matching the first open curly bracket `{`
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
				return abort("Wrong syntax, '}' without a matching '{'", Void);
			}
			stack.pop().unwrap();
			stop = clos_br + 1;

		// the open bracket is first
		} else if open_br < clos_br {
			stack.push('{');
			stop = open_br + 1;
		}

		res += stop;

		file_data = &file_data[stop..];

		if stack.is_empty() {
			break;
		}
	}

	res += file_data.find('\n').unwrap_or(0);

	return res;
}

/// returns the index of the smallest element in the given vector
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

/// reads and writes to file the input file untile the 'template<...>' keyword is found
/// returns the parsed templated 
fn consume_till_template(file_data: &str, mut output_file: &fs::File) -> (usize, usize) {
	let tmp = format!(r"{}\s?<.*>.*\n", TEMPLATE_DECLARATION_KEYWORD);
	let re = Regex::new(&tmp).unwrap(); // no need to take care of any error, the pattern is valid and too small to fail

	let res = re.find(file_data);
	if res == None {
		output_file.write(file_data.as_bytes()).expect("Failed Write");
		return abort("Template declaration not found. Skipping", Void);
	} else {
		let m = res.unwrap();
		return (m.start(), m.end());
	}
}

/// given a slice containing a function, replaces all of the template types with the target types
fn complete_template(mut file_data: &str, template_names: &Vec<String>, target_types: &Vec<String>, mut output_file: &fs::File) {
	/*
	This function, to work nicely, needs a lexer, a tokenizer, a CFG decoder, however the fuck is called the thing in the compiler that recognizes keywords, operators, and names.
	But I ain't gonna do that.
	Not even gonna fucking try. C isn't super diffucult (except things like function pointers typedefs) but still, there are multiple standars and dialects for each compiler, thus No.

	I'm gonna assume that no one is feeding this tool minified C code (if such a thing even exists), so i will assume that all template types have at least a space after them
	(thing I'm pretty sure is obligatory) and check for opening and closign brackets, '{' and '}',

	This means that anything inside comments will not be treated as such, so you might fuck up the function end detection with brackets inside comments, and the templated type will be replaced inside of them.

	good luck
	*/

	// records all of the position, to figure out which comes first
	// Vec<[start, end, template type index]>
	let mut positions: Vec<[usize; 3]> = Vec::new();

	// for all template types
	for i in 0..template_names.len() {
		let T = &template_names[i];

		let tmp = format!(r"\W({})\W|(##{})|^({})", T, T, T);
		let re = Regex::new(&tmp).unwrap(); // no need to take care of any error, the pattern is valid and too small to fail

		// for all the matches
		for c in re.captures_iter(file_data) {
			let mut cap = c.get(1);
			if cap == None {
				cap = c.get(2);
			}

			if cap == None {
				cap = c.get(3);
			}

			if cap == None {
				println!("The regex return an empty capture, somehow.");
				exit(1);
			}

			let m = cap.unwrap();

			positions.push([m.start(), m.end(), i]);
		}
	}

	// this i stecnically useless
	// I need the matches to be in order to easily be able to write till the match, write the tartget type, and continue
	// and the regex matches from the start of the string to the end, so in order.
	// also I'm sorting array of non intersecting values (cuz of how the regex works) and have no idea how it works
	// but I'll keep it here for now
	positions.sort();
	let mut stop: usize = 0;

	// positions  = Vec<[start, end]>
	// just like slices, start is included, end is excluded
	for span in positions {
		output_file.write(file_data[stop..span[0]].as_bytes()).expect("Failed Write");

		output_file.write(template_names[span[2]].as_bytes()).expect("Failed Write");

		stop = span[1]
	}
	file_data = &file_data[stop..];
	// print the rest of the chunk
	output_file.write(file_data.as_bytes()).expect("Failed Write");
}

/// Given the template declaration (`template <T, U, V, ...>`)
/// returns the template types (`T`, `U`, `V`) as a `Vec` of owned `String`s
fn parse_template_declarations(file_data: &str) -> Vec<String> {
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
	let target_types = &args[2..].to_vec();

	let file = match fs::read_to_string(target_filename) {
		| Ok(dt) => dt,
		| Err(e) => abort(&format!("Could not read from the file {target_filename}"), Some(e)),
	};

	let mut output_file = fs::File::create("tl.out").expect("Failed Create");

	let (start, end) = consume_till_template(&file[0..], &output_file);

	// the TEMPLATE_DECLARATION_KEYWORD might not have been found
	let template_decls = parse_template_declarations(&file[start..end]);



	return;
	/*

		// WTH Rust WTH
		let mut file_data = &file[0..];
		let mut line_num: usize = 0; // needed to diagnostics

		// reading line by line
		let mut nl: usize = 0;
		loop {

			match file_data.find("\n") {
				| Some(val) => nl = val,
				| None => break,
			};

			let line = &file_data[..nl];

			line_num += 1;

			file_data = &file_data[(nl + 1)..];
			if line.contains(TEMPLATE_DECLARATION_KEYWORD) {
				found = TRUE;
				break;
			}
			output_file.write(line.as_bytes()).expect("Failed Write");
		}


		let dc_len = template_decls.len();
		let tt_len = target_types.len();
		if tt_len != dc_len {
			abort::<i32, Dummy>(&format!("The target types ({tt_len}) do not match the number of template placeholders ({dc_len}) at line {line_num}"), Void);
		}

		complete_template(&file_data, &template_decls, target_types, &output_file);
	*/
}
