///
/// Author: m_remon
///
/// Templetizer
/// A simple rust program to convert a C file with templates into a compilable C files with the templates replaced with actual types
///
/// Naming:
///   Template: the placeholder type used inside the template declaration and on the program itself
///   Template declaration: the line `template <T, U, V, ...>` that declare for the first time a template in the program
///
/// Usage:
///   templetizer -i input.ct -o output.c -t T1, T2, T3, ...
/// where:
///   -i denotes the input file, a C file that makes use of a simple template syntax (explained below)
///   -o denotes the output file, a normal C file compilable by a combiler
///   -t denotes the start of a type sequence T1, T2, T3, ..., they are the types used to replace the templates with
///
/// Outside Behaviour:
///   To know which types are templates the program searches for a C++ like template declaration in the file, only after that it will attempt to
///   replace the Templates.
///   The templates are replace by the given types resoecting the order, if the call is `templatizer input.ct int, double, Person` and
///   the template declaration is `template <T, U, V>` this is the association `T = int`,`U = double`, and`V = Person`
///   The tool is quite stupid, it is not context aware as it uses a simple regex to detect Templates in most normal circumstances
///   but it cannot detect if it is trying to replace a template inside a comment, and I'm too lazy to fix this
///
/// Inner Behaviour:
///   To avoid replacing strings in memory I've done some gymnastics with slices when i have to write to file
///   I copy the input file to the output file until the byte before the string to replace (or ignore in the case of the template)
///   write the actual type, and continue with the input file. This is carried out till the end of the input
///
/// Syntax:
///   `template<T, U, V, ...>` ONCE inside the input file, this is needed to know how many templates there are and their names
///   `T` or any equivalent template used as a type, the regex search for the it surrounded by non words (a word being everythin alphanumeric + _)
///   `#T#` a special syntax to glue the replaced type to any string near the `#` character
///
/// Upgrades:
///   Better Cli interface
///   Comment detection: Don't replace anything inside a comment
///   File watching: keep watching the input file (evey x sec) to transpile it if it changes
use std::env; // to collect args
use std::fs; // to manages files
use std::io::Write; // to write to files
use std::usize;
use std::vec; // for unambiguas byte offsets

extern crate regex;
use regex::Regex; // searching inside the file

// constants
const TEMPLATE_DECLARATION_KEYWORD: &str = "template";

const CLI_SWITCHED: [&str; 3] = ["-i", "-o", "-t"];

// This struct and the relative function are an excercise in 'breaking' the type system in doing what i want
// It's not very important, dw
struct Dummy {}

impl std::fmt::Debug for Dummy {
	fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
		return Ok(());
	}
}

const VOID: Option<Dummy> = Option::<Dummy>::None;

/// The whole point of this function is to exit the program with an error printed, I later found out about `.expect(...)`
/// but this is still usefull in some occasions
fn abort<T, U: std::fmt::Debug>(s: &str, err: Option<U>) -> T {
	match err {
		| Some(e) => eprintln!("Aborting: {s}\n\t{e:#?}"),
		| None => eprintln!("Aborting: {s}"),
	}
	std::process::exit(1);
}

/// reads and writes to the output file the input file until the 'template<...>' keyword is found
/// returns the template position and span
fn consume_till_template(file_data: &str, output_file: &mut Box<dyn Write>) -> (usize, usize) {
	let tmp = format!(r"{TEMPLATE_DECLARATION_KEYWORD}\s?<.*>");
	let re = Regex::new(&tmp).unwrap(); // no need to take care of any error, the pattern is valid and too small to fail

	let res = re.find(file_data);

	// simply outut the input, no work to do
	if res == None {
		output_file.write(file_data.as_bytes()).expect("Failed Write");
		return abort("Template declaration not found. Skipping", VOID);
	} else {
		let m = res.unwrap();
		return (m.start(), m.end());
	}
}

/// reads and writes to the output file the input file, if a template is found, it is replaced with the corresponding target type
fn consume_templates(mut file_data: &str, template_names: &Vec<String>, target_types: &Vec<&String>, output_file: &mut Box<dyn Write>) {
	/*
	This function, to work nicely, would need a lexer, a tokenizer, a CFG decoder, however the fuck is called the thing in the compiler that recognizes keywords, operators, and names.
	But I ain't gonna do that.
	Not even gonna fucking try. C isn't super diffucult (except things like function pointers typedefs) but still, there are multiple standars and dialects for each compiler, thus No.

	I have no idea which edge case I'm missing but oh well, I'll burn that burn when I'll get there.

	I know that comments are not treated as such, so it might happen that a rendom `T` will get detected and promptly substituted
	But that's a feature if you ask me, templated comments, a revoluton in documentation generation

	good luck
	*/

	// records all of the position, to figure out which comes first
	// Vec<[start, end, template type index]>
	let mut positions: Vec<[usize; 3]> = Vec::new();

	// for all template types
	for i in 0..template_names.len() {
		let t = &template_names[i];

		// capturing:
		//    a T between non words
		//    a T between two hashes ##
		let tmp = format!(r"(#{t}#)|\W({t})\W");
		let re = Regex::new(&tmp).unwrap(); // no need to take care of any error, the pattern is valid and too small to fail

		// for all the matches
		for c in re.captures_iter(file_data) {
			// the capture are numbered, and since I'm using a disjuncton in the regex only one the 2 has actually matched
			let mut cap = c.get(1);
			if cap == None {
				cap = c.get(2);
			}

			if cap == None {
				abort::<i32, Dummy>("The regex returned an empty capture, somehow.", VOID);
			}

			let m = cap.unwrap();

			positions.push([m.start(), m.end(), i]);
		}
	}

	// FIXME: possible error here
	// I need the matches to be in order to easily be able to write till the match, write the tartget type, and continue
	// and the regex matches from the start of the string to the end, so in order.
	// but if there are multiple template types the regex need to run for each one, thus possibly producing unirdered matches
	// also I'm sorting array of non intersecting values (cuz of how the regex works) and have no idea how it works
	positions.sort();

	let mut stop: usize = 0;

	// positions  = Vec<[start, end, index]>
	// just like slices, start is included, end is excluded
	for span in positions {
		// write till before the span
		output_file.write(file_data[stop..span[0]].as_bytes()).expect("Failed Write");

		// write the replacement type
		output_file.write(target_types[span[2]].as_bytes()).expect("Failed Write");

		// skip till the end of the span
		stop = span[1]
	}
	// advance to the end of the last capture
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
		| None => abort("Invalid template syntax, missing '<'", VOID),
	};

	let clos_br = match file_data.find(">") {
		| Some(val) => val,
		| None => abort("Invalid template syntax, missing '>'", VOID),
	};

	let slice = &file_data[open_br..clos_br];

	let parts = slice.split(",");

	for p in parts {
		res.push(p.trim().to_owned());
	}

	return res;
}

/// parses the arguments given and the cli swtches within
fn parse_args(args: &Vec<String>) -> (&str, &str, Vec<&String>) {
	let mut input_path = "";
	let mut output_path = "";
	let mut target_types = vec![];
	let mut i: usize = 1;

	// index based for to skip args if needed
	loop {
		if i >= args.len() {
			break;
		}

		// input file
		if "-i" == args[i] {
			i += 1;
			if i >= args.len() {
				abort::<i32, Dummy>("Missing input file path", VOID);
			}
			input_path = &args[i];

		// output file
		} else if "-o" == args[i] {
			i += 1;
			if i >= args.len() {
				abort::<i32, Dummy>("Missing output file path", VOID);
			}
			output_path = &args[i];

		// target types
		} else if "-t" == args[i] {

			// everything until another cli switch
			for k in &args[i + 1..] {
				if CLI_SWITCHED.contains(&k.as_str()) {
					break;
				}
				target_types.push(k);
			}

			i += target_types.len();

		// wrong
		} else {
			let v = &args[i];
			abort::<i32, Dummy>(&format!("'{v}' Unrecognized option"), VOID);
		}

		i += 1;
	}

	if input_path == "" {
		abort::<i32, Dummy>("Missing input file path", VOID);
	}

	if target_types == vec![""] {
		abort::<i32, Dummy>("No types to replace the template with", VOID);
	}

	return (input_path, output_path, target_types);
}

fn main() {
	// --------------------------------------------------------------------------------------------
	// checking cli args
	// --------------------------------------------------------------------------------------------

	let args: Vec<String> = env::args().collect();

	let (i_file, o_file, target_types) = parse_args(&args);

	// --------------------------------------------------------------------------------------------
	// creating and reading files
	// --------------------------------------------------------------------------------------------

	let target_filename: &str = &args[1];

	let file = match fs::read_to_string(i_file) {
		| Ok(dt) => dt,
		| Err(e) => abort(&format!("Could not read from the file {target_filename}"), Some(e)),
	};

	// traits are FUN :)))))) 
	let mut output_file = Box::new(std::io::stdout()) as Box<dyn Write>;

	if o_file != "" {
		output_file = Box::new(fs::File::create(o_file).expect("Failed Create")) as Box<dyn Write>;
	}

	// --------------------------------------------------------------------------------------------
	// Searching for template declaration
	// --------------------------------------------------------------------------------------------

	let (start, end) = consume_till_template(&file[0..], &mut output_file);

	let template_decls = parse_template_declarations(&file[start..end]);

	let dc_len = template_decls.len();
	let tt_len = target_types.len();
	if tt_len != dc_len {
		abort::<i32, Dummy>(&format!("The number of types given via cli ({tt_len}) do not match the number of template placeholders ({dc_len}) present in the file."), VOID);
	}

	// --------------------------------------------------------------------------------------------
	// Replacing the templates
	// --------------------------------------------------------------------------------------------

	consume_templates(&file[end..], &template_decls, &target_types, &mut output_file);
}
