use std::sync::Barrier;
use std::thread;

use measured::label::StaticLabelSet;
use measured::metric::histogram::Thresholds;
use measured::{CounterVec, Exemplar, ExemplarHistogram, FixedCardinalityLabel};

#[test]
fn histogram_snapshots_stay_consistent_during_concurrent_observations() {
    let histogram = ExemplarHistogram::with_metadata(Thresholds::with_buckets([1.0]));
    let start = Barrier::new(5);
    let done = Barrier::new(5);
    let mut snapshots = Vec::new();
    let mut checkpoints = Vec::new();
    thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(|| {
                for _ in 0..8 {
                    start.wait();
                    for _ in 0..100 {
                        histogram.observe_with_exemplar(1.0, Exemplar::new());
                    }
                    done.wait();
                }
            });
        }
        for _ in 1..=8 {
            start.wait();
            for _ in 0..100 {
                snapshots.push(histogram.sample());
            }
            done.wait();
            checkpoints.push(histogram.sample().count());
        }
    });
    // Assert after joining so a failed assertion cannot strand writers at a barrier.
    for sample in snapshots {
        assert_eq!(sample.sum, sample.count() as f64);
        assert_eq!(sample.inf, 0);
    }
    assert_eq!(checkpoints, [400, 800, 1200, 1600, 2000, 2400, 2800, 3200]);
    assert_eq!(
        histogram.get_metric().exemplars()[0]
            .as_ref()
            .unwrap()
            .value(),
        1.0
    );
}

#[test]
fn sparse_removal_preserves_every_increment_in_live_or_removed_state() {
    #[derive(Clone, Copy, FixedCardinalityLabel)]
    #[label(singleton = "worker")]
    enum Worker {
        Shared,
    }

    let counters = CounterVec::<StaticLabelSet<Worker>>::sparse();
    let id = counters.with_labels(Worker::Shared);
    let start = Barrier::new(5);
    let removed = thread::scope(|scope| {
        for _ in 0..4 {
            let start = &start;
            let counters = &counters;
            scope.spawn(move || {
                start.wait();
                for _ in 0..1000 {
                    counters.inc(Worker::Shared);
                }
            });
        }
        let removal = scope.spawn(|| {
            start.wait();
            let mut removed = 0;
            for _ in 0..1000 {
                if let Some(metric) = counters.remove_metric(id) {
                    removed += metric.get();
                }
            }
            removed
        });
        removal.join().unwrap()
    });
    assert_eq!(removed + counters.get(Worker::Shared), 4000);
}
