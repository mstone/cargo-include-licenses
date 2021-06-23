## What does this program do?
It goes through the local copies of the dependencies of a project  
(or, to be more precise, the packages output by `cargo metadata`) and looks for potential license files:  
- Files than have a path containing "`LICENSE|COPYRIGHT|NOTICE|AUTHORS|COPYING`" (or in lowercase)
- Files that have a path containing "`README`", "`.txt`" or "`.md`" and include a line with one of the words mentioned above

These are then copied (while preserving the directory structure) into a user-defined directory.

Of course, there can be both false negatives and false positives, use it at your own risk.

## Building
Due to the usage of `once_cell`, it currently requires Rust nightly.  
Otherwise it can be built with `cargo`, just like usual.
