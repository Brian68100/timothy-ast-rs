A simple, fully featured virtual machine (VM) written in Rust

To run it as a shell, on the command line, type:

    $ timothy-ast-rs

To run a script, on the command line, type:

    $ timothy-ast-rs script-name.timothy

To run with command line args, type:

    $ timothy-ast-rs -- [arg ...]

        or

    $ timothy-ast-rs script-name.timothy [arg ...]

        or

    $ timothy-ast-rs script-name.timothy -- [arg ...]


To run it embedded within another program from within Rust,
type in a text editor:

    use timothy_ast_rs::parser::Parser;
        
    fn main() {
        parser = Parser::new();
        let result = parser.parse_text("println(\"Hello, world!\"");
        switch result {
            Err(_) => println!("Error occurred!");
            _ => return;
        }
    }



