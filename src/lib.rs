mod fetch;
mod print;
mod worker;

use std::rc::Rc;

use chrono::{DateTime, Utc};
pub use fetch::Fetch;
use fetch::RealFetcher;
use once_cell::sync::OnceCell;
pub use print::Print;
use print::{FilePrinter, RealPrinter};
use worker::Worker;

// This will be generated by the macro passed on what the user wrote
pub trait DependencyContainer {
    fn date_time(&self) -> DateTime<Utc>;

    fn printer(&self) -> impl Print;
    fn fetcher(&self) -> impl Fetch;
    fn worker(&self) -> Worker<impl Fetch, impl Print>;

    fn new_scope(&self) -> Self;
}

pub struct DependencyContainerImpl {
    file: Option<String>,
    printer: Rc<OnceCell<Box<dyn Print>>>,
    fetcher: Rc<OnceCell<RealFetcher>>,
}

// Has all the helper methods and likely to be the input of the macro
// Ie. this is what the user will write... and only this
impl DependencyContainerImpl {
    // Show that external runtime config is possible
    pub fn new(file: Option<String>) -> Self {
        Self {
            file,
            printer: Rc::new(OnceCell::new()),
            fetcher: Rc::new(OnceCell::new()),
        }
    }

    // To be able to store whatever this creates on in the `Self::fetcher` field
    // without introducing generics on `Self`, this needs to return the concrete type
    // ... I don't see a way around this
    fn create_fetcher(&self, date_time: DateTime<Utc>) -> RealFetcher {
        println!("Scoped Fetcher");
        RealFetcher::new(date_time)
    }

    // Example of something that chooses the concrete type at runtime. So need to `Box` it
    fn create_printer(&self) -> Box<dyn Print> {
        println!("Singleton Printer");
        if let Some(file) = &self.file {
            Box::new(FilePrinter::new(file.clone()))
        } else {
            Box::new(RealPrinter::new())
        }
    }

    // Just showing that sometimes a dependency is not a trait at all
    fn create_worker(
        &self,
        fetcher: impl Fetch,
        printer: impl Print,
    ) -> Worker<impl Fetch, impl Print> {
        Worker::new(fetcher, printer)
    }
}

// This implements the trait and is what the macro should generate
// The macro will also generate the trait definition
impl DependencyContainer for DependencyContainerImpl {
    // Boilerplate for making a transient lifetime dependency
    fn date_time(&self) -> DateTime<Utc> {
        println!("Transient DateTime");
        Utc::now()
    }

    // Boilerplate for making a scoped lifetime dependency
    fn fetcher(&self) -> impl Fetch {
        let fetcher = self
            .fetcher
            .get_or_init(|| self.create_fetcher(self.date_time()));

        fetcher
    }

    // Boilerplate for making a singleton lifetime dependency
    // Is same as scoped
    fn printer(&self) -> impl Print {
        let printer = self.printer.get_or_init(|| self.create_printer());

        printer
    }

    // Another transient lifetime that does not use a trait type
    fn worker(&self) -> Worker<impl Fetch, impl Print> {
        self.create_worker(self.fetcher(), self.printer())
    }

    // A new scope should copy all the internal options and any singletons that might already exist
    // But scoped lifetime dependencies should be reset. Aka make new Rc<OnceCell<>> for them
    fn new_scope(&self) -> Self {
        Self {
            file: self.file.clone(),
            printer: self.printer.clone(),
            fetcher: Rc::new(OnceCell::new()),
        }
    }
}

// Allow the DI container to be passed around easily to sub functions
impl Clone for DependencyContainerImpl {
    fn clone(&self) -> Self {
        Self {
            file: self.file.clone(),
            printer: self.printer.clone(),
            fetcher: self.fetcher.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::{worker::Worker, DependencyContainer, Fetch, Print};
    use chrono::prelude::*;

    struct MockFetcher;

    impl Fetch for MockFetcher {
        fn fetch(&self) -> String {
            "MockFetcher".to_string()
        }
    }

    struct MockPrinter {
        data: Option<String>,
    }

    impl Print for Rc<RefCell<MockPrinter>> {
        fn print(&self, data: &str) {
            self.borrow_mut().data.replace(data.to_string());
        }
    }

    #[derive(Clone)]
    pub struct MockDependencyContainer {
        printer: Rc<RefCell<MockPrinter>>,
    }

    impl DependencyContainer for MockDependencyContainer {
        fn date_time(&self) -> DateTime<Utc> {
            Utc.with_ymd_and_hms(2024, 5, 14, 22, 0, 0).unwrap()
        }

        fn printer(&self) -> impl Print {
            self.printer.clone()
        }

        fn fetcher(&self) -> impl Fetch {
            MockFetcher
        }

        fn worker(&self) -> Worker<impl Fetch, impl Print> {
            Worker::new(self.fetcher(), self.printer())
        }

        fn new_scope(&self) -> Self {
            self.clone()
        }
    }

    #[test]
    fn test_worker() {
        let mdm = MockDependencyContainer {
            printer: Rc::new(RefCell::new(MockPrinter { data: None })),
        };

        mdm.worker().work();

        assert_eq!(mdm.printer.borrow().data.as_ref().unwrap(), "MockFetcher");
    }
}
