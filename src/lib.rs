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

pub trait DependencyContainer {
    fn worker(&self) -> Worker<impl Fetch, impl Print>;

    fn new_scope(&self) -> Self;
}

pub struct DependencyContainerImpl {
    file: Option<String>,
    printer: Rc<OnceCell<Box<dyn Print>>>,
    fetcher: Rc<OnceCell<RealFetcher>>,
}

impl DependencyContainerImpl {
    pub fn new(file: Option<String>) -> Self {
        Self {
            file,
            printer: Rc::new(OnceCell::new()),
            fetcher: Rc::new(OnceCell::new()),
        }
    }

    // Transient lifetime
    fn date_time(&self) -> DateTime<Utc> {
        println!("Transient DateTime");
        Utc::now()
    }

    // Scoped lifetime
    fn fetcher(&self) -> impl Fetch + '_ {
        let fetcher = self
            .fetcher
            .get_or_init(|| self.create_fetcher(self.date_time()));

        fetcher
    }

    fn create_fetcher(&self, date_time: DateTime<Utc>) -> RealFetcher {
        println!("Scoped Fetcher");
        RealFetcher::new(date_time)
    }

    // Singleton lifetime
    fn printer(&self) -> impl Print + '_ {
        let printer = self.printer.get_or_init(|| self.create_printer());

        printer
    }

    fn create_printer(&self) -> Box<dyn Print> {
        println!("Singleton Printer");
        if let Some(file) = &self.file {
            Box::new(FilePrinter::new(file.clone()))
        } else {
            Box::new(RealPrinter::new())
        }
    }

    // Extra
    fn create_worker(
        &self,
        fetcher: impl Fetch,
        printer: impl Print,
    ) -> Worker<impl Fetch, impl Print> {
        Worker::new(fetcher, printer)
    }
}

impl DependencyContainer for DependencyContainerImpl {
    fn worker(&self) -> Worker<impl Fetch, impl Print> {
        self.create_worker(self.fetcher(), self.printer())
    }

    fn new_scope(&self) -> Self {
        Self {
            file: self.file.clone(),
            printer: self.printer.clone(),
            fetcher: Rc::new(OnceCell::new()),
        }
    }
}

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
        fn worker(&self) -> Worker<impl Fetch, impl Print> {
            let fetcher = MockFetcher;
            let printer = self.printer.clone();

            Worker::new(fetcher, printer)
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
