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

pub trait DependencyContainerTrait {
    fn worker(&self) -> Worker<impl Fetch, impl Print>;

    fn new_scope(&self) -> Self;
}

pub struct DependencyContainer {
    file: Option<String>,
    printer: Rc<OnceCell<Box<dyn Print>>>,
    fetcher: OnceCell<RealFetcher>,
}

impl DependencyContainer {
    pub fn new(file: Option<String>) -> Self {
        Self {
            file,
            printer: Rc::new(OnceCell::new()),
            fetcher: OnceCell::new(),
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
            .get_or_init(|| self.fetcher_manual(self.date_time()));

        fetcher
    }

    fn fetcher_manual(&self, date_time: DateTime<Utc>) -> RealFetcher {
        println!("Scoped Fetcher");
        RealFetcher::new(date_time)
    }

    // Singleton lifetime
    fn printer(&self) -> impl Print + '_ {
        let printer = self.printer.get_or_init(|| self.printer_manual());

        printer
    }

    fn printer_manual(&self) -> Box<dyn Print> {
        println!("Singleton Printer");
        if let Some(file) = &self.file {
            Box::new(FilePrinter::new(file.clone()))
        } else {
            Box::new(RealPrinter::new())
        }
    }

    // Extra
    fn worker_manual(
        &self,
        fetcher: impl Fetch,
        printer: impl Print,
    ) -> Worker<impl Fetch, impl Print> {
        Worker::new(fetcher, printer)
    }
}

impl DependencyContainerTrait for DependencyContainer {
    fn worker(&self) -> Worker<impl Fetch, impl Print> {
        self.worker_manual(self.fetcher(), self.printer())
    }

    fn new_scope(&self) -> Self {
        self.clone()
    }
}

impl Clone for DependencyContainer {
    fn clone(&self) -> Self {
        Self {
            file: self.file.clone(),
            printer: self.printer.clone(),
            fetcher: OnceCell::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::{worker::Worker, DependencyContainerTrait, Fetch, Print};

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
    pub struct MockDependencyManager {
        printer: Rc<RefCell<MockPrinter>>,
    }

    impl DependencyContainerTrait for MockDependencyManager {
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
        let mdm = MockDependencyManager {
            printer: Rc::new(RefCell::new(MockPrinter { data: None })),
        };

        mdm.worker().work();

        assert_eq!(mdm.printer.borrow().data.as_ref().unwrap(), "MockFetcher");
    }
}
