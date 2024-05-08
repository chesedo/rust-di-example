use crate::{Fetch, Print};

pub struct Worker<F, P> {
    fetcher: F,
    printer: P,
}

impl<F, P> Worker<F, P>
where
    F: Fetch,
    P: Print,
{
    pub fn new(fetcher: F, printer: P) -> Self {
        Self { fetcher, printer }
    }

    pub fn work(&self) {
        let data = self.fetcher.fetch();
        self.printer.print(&data);
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;

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

    #[test]
    fn test_work() {
        let fetcher = MockFetcher;
        let printer = Rc::new(RefCell::new(MockPrinter { data: None }));

        let worker = Worker::new(fetcher, printer.clone());
        worker.work();

        assert_eq!(printer.borrow().data.as_ref().unwrap(), "MockFetcher");
    }
}
