use chrono::{DateTime, DurationRound, TimeDelta, Utc};

pub trait Fetch {
    fn fetch(&self) -> String;
}

pub struct RealFetcher {
    date_time: DateTime<Utc>,
}

impl RealFetcher {
    pub fn new(date_time: DateTime<Utc>) -> Self {
        println!("RealFetcher");
        Self {
            date_time: date_time
                .duration_trunc(TimeDelta::days(1))
                .expect("to truncate date"),
        }
    }
}

impl Fetch for RealFetcher {
    fn fetch(&self) -> String {
        self.date_time.to_rfc3339()
    }
}

impl<F> Fetch for &F
where
    F: Fetch,
{
    fn fetch(&self) -> String {
        (*self).fetch()
    }
}
