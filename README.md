A simple, fully featured virtual machine (VM) written in Rust

To run it as a shell, on the command line, type:

    $ timothy-ast-rs

To run a script, on the command line, type:

    @ timothy-ast-rs script-name.timothy

To run with command line args, type:

    $ timothy-ast-rs -- [arg ...]

        or

    $ timothy-ast-rs script-name.timothy [arg ...]

        or

    $ timothy-ast-rs script-name.timothy -- [arg ...]


To run it embedded within another program from within Rust, type:

    use timothy-ast-rs::parser::Parser

        and
        
    fn main() {
        parser = Parser::new();
        TResult result = parser.parse_text("println(\"Hello, world!\"");
        match result {
    }



