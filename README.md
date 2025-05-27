```
  ______                     __     __  _                
 /_  __/__  ____ ___  ____  / /__  / /_(_)___  ___  _____
  / / / _ \/ __ `__ \/ __ \/ / _ \/ __/ /_  / / _ \/ ___/
 / / /  __/ / / / / / /_/ / /  __/ /_/ / / /_/  __/ /    
/_/  \___/_/ /_/ /_/ .___/_/\___/\__/_/ /___/\___/_/     
                  /_/                                    
```

# Templetizer

A cli tool to transpile templated `C` source files into compilable `C` files

## Why?

I prefer using `C` over `C++`, especially after the `C23` standard was released, but this means I don't have access to `C++`'s `templates`
So if I need to use a data structure either I structure it to take `void*`, element sizes, and do a lot of casting or I'm forced to copy paste the entire code changing the type it operates on, forcing me to keep multiple quasi-identical files updated if I find any bugs in them.

Also this [talk](https://www.youtube.com/watch?v=rX0ItVEVjHc&t=4290s) from Mike acton

So I created this.

### Why Rust?

I don't really enjoy string manipulation (in `C`) and I wanted to get familiar with `Rust`
Other than the fact that it being quite performant is quite good.

## Modus Operandi

I came into this project with the objective of avoiding 'in String replacement'. (i.e. replacing a part of a string with a another, usually longer, string)
These operation necessitates the relocation of a part (in this case it may be a big part of the file) of the string, which may cause additional allocations.

So, instead of doing that for each time I have to replace a pattern, the program:
- Writes to the output till the beginning of the pattern (`o_file.write(i_file[..start])`)
- Writes the replacement string (`o_file.write(target_type)`)
- Writes the rest of the input file until the next pattern (`o_file.write(i_file[stop..start])`)


Logically it performs these operations
- Reads and parses the cli args
- Executes the templetizer on the file given
- If asked to watches for changes on the input file to trigger the templetizer again

## Features

The tool has only one feature
- Transpile `C` files with templates into normal `C` files

Additionally it can watch the input file for changes to instantly templetize it.

## Usage

```
Usage:
templetizer -i <filename> -t <target types> [-o <filename>] [--watch]
General options

	-i <filename>         specify the input file
	-t <list>             a space separated list of target files, it stop at the specification of another argument or at the end of the line
	-o <filename>         specify the output file to write the transpiled code to, else stdout will be used
	--watch               keep watching the input file for changes, if they occurr execute the templetizer again
	-h, --help            print this screen

```

The input file is a `C` file with `C++` style templates, (the syntax is made up) The tool will search for a single template declaration `template<T, U, V, ...>`
After that it will replace every instance of the declared template types found in the program, when used as types, with the corresponding target type given via the cli
The template <-> target type association is based on the order of the target types in the command call and in the template declaration.

for example:
With `templetizer -i hash.ct -t user size_t ...`
and in `hash.ct` the declaration is `template<K, V>`
The association is
- `K` = `user`
- `V` = `size_t`

Additionally the program recognizes `#T#` (where `T` is any template type) and will replace it whole with the associated target type.
This is intended to be used for renaming structs or functions based on the type they operate on.
Nothing stops you from combining types together with something like `#T##K#` though.


## Caveats

`C++` templates have the advantage of being 'compiled' when they are called, having available the type defition necessary for their correctness
This templetizer can't do that, but can still get away with it by exploting what counts as a `target` type
The problem really arises if you need a user defined type in a file (header or not). You could do something like `#include "T.h"` and it would work, but it might also result in `#include "int.h"` if called with `int` as target type.
`#include "int.h"` does not compile.

The solution I've found is to use a template type for the sole purpose of being an optional include:
```c
template <K, V, I>

#include <stddef.h>
#include <stdio.h>

#include "utils.h"
I
```

With this setup a call with
`templetizer -i hash.ct -t user size_t \#include\ \"types.h\"` (with backspaces to escape quotes and spaces)
Will result in the include being added

```c
template <K, V, I>

#include <stddef.h>
#include <stdio.h>

#include "utils.h"
#include "types.h"
```

And with this call
`templetizer -i hash.ct -t user size_t \ `
Will result in an additional blank line, preventing empty includes
```c
template <K, V, I>

#include <stddef.h>
#include <stdio.h>

#include "utils.h"

```
