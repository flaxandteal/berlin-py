use std::time::Instant;

use priority_queue::PriorityQueue;
pub use rayon;
use tracing::info;
pub use ustr;
use ustr::{Ustr, UstrMap};

use crate::json_decode::Location;

pub mod json_decode;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

#[derive(Default)]
pub struct CodeBank {
    pub all: UstrMap<Location>,
}

pub fn search(cb: &CodeBank, search_term: String) {
    let start = Instant::now();
    let mut pq: PriorityQueue<Ustr, i64> = PriorityQueue::new();
    for (key, loc) in cb.all.iter() {
        let search_score: i64 = (loc.search(&search_term) * 1000.) as i64;
        pq.push(*key, search_score);
    }
    let first = pq.pop();
    info!("First result {:?}", first);
    if let Some(first) = first {
        info!("{:#?}", cb.all.get(&first.0));
    }
    info!("Search took {:.2?}", start.elapsed())
}
