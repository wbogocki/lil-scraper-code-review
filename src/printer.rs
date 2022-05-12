use comfy_table::{Cell, Color, Table};

pub trait Printer {
    // NOTE(Wojciech): This will work... but I'd avoid doing this if possible. The exact reason and problem and the
    // issue I solved with this is very specific so we'll do it on the call.

    /*

    Where this got me in trouble:

    fn spawner::<T: Trait>() {
        let spawn_params = ...; // Only known inside spawner() and not to the caller.

        let mut things = Vec::new();
        for t in 0..10 {
            things.push(T::new(&spawn_params));
        }
    }

    fn caller() {
        spawner::<Thing>();
    }

    // What if caller() needs to pass some arguments to ThingType?
    // Now you need to add them to Trait::new(), pass them as arguments to spawner() for each Thing that implements the Trait.

    // Unsure about this but it's possible that the creation place of Thing (the specific type) and the arguments for it tend to be
    // available at the same place so by moving them down the callstack we just made things less obvious and more complicated for
    // no reason at all.

    */

    // fn new() -> Self
    // where
    //     Self: Sized;
    fn success(&mut self, target: &str, result: &str);
    fn error(&mut self, target: &str, result: &str);
    fn finish(&self);
}

pub struct TablePrinter {
    table: Table,
}

impl TablePrinter {
    pub fn new() -> Self {
        let mut table = Table::new();
        table.set_header(vec!["Target", "Result"]);
        TablePrinter { table }
    }
}

impl Printer for TablePrinter {
    fn success(&mut self, target: &str, result: &str) {
        self.table.add_row(vec![target, result]);
    }

    fn error(&mut self, target: &str, result: &str) {
        self.table
            .add_row(vec![Cell::new(target), Cell::new(result).bg(Color::Red)]);
    }

    fn finish(&self) {
        println!("{}", self.table);
    }
}

pub struct TextPrinter {}

impl TextPrinter {
    pub fn new() -> Self {
        TextPrinter {}
    }
}

impl Printer for TextPrinter {
    fn success(&mut self, target: &str, result: &str) {
        println!("{}, {}", target, result);
    }

    fn error(&mut self, target: &str, result: &str) {
        println!("{}, {}", target, result);
    }

    fn finish(&self) {}
}
