A simple, fully featured virtual machine (VM) written in Rust

To run it as a shell, on the command line, type:

    $ timothy

To run a script, on the command line, type:

    $ timothy script-name.timothy

To run with command line args, type:

    $ timothy -- [arg ...]

        or

    $ timothy script-name.timothy [arg ...]

        or

    $ timothy script-name.timothy -- [arg ...]


To run it embedded within another program from within Rust,
type in a text editor:

    use timothy::parser::Parser;
        
    fn main() {
        let parser = Parser::new();
        let result = parser.parse_text("println(\"Hello, world!\"");
        switch result {
            Err(_) => println!("Error occurred!");
            _ => return;
        }
    }

To run a script embedded within another program from within Rust,
type in a text editor

    use timothy::parser::Parser;

    fn main() {
        let parser = Parser::new();
        let result = parser.parse_file("my_script.timothy");
        switch result {
            Err(_) => println!(Error occurred!");
            _ => return;
        }
    }




