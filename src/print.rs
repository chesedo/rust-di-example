pub trait Print {
    fn print(&self, data: &str);
}

// impl Print for Rc<dyn Print> {
//     fn print(&self, data: &str) {
//         self.as_ref().print(data);
//     }
// }

impl Print for &Box<dyn Print> {
    fn print(&self, data: &str) {
        self.as_ref().print(data);
    }
}

pub struct RealPrinter;

impl Print for RealPrinter {
    fn print(&self, data: &str) {
        println!("{}", data);
    }
}

impl RealPrinter {
    pub fn new() -> Self {
        println!("RealPrinter");
        Self
    }
}

pub struct FilePrinter {
    file: String,
}

impl Print for FilePrinter {
    fn print(&self, data: &str) {
        std::fs::write(&self.file, data).expect("to write file");
    }
}

impl FilePrinter {
    pub fn new(file: String) -> Self {
        println!("FilePrinter");
        Self { file }
    }
}
