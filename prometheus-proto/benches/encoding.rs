use divan::black_box;
use divan::Bencher;
use lasso::{Spur, ThreadedRodeo};
use measured::{CounterVec, LabelGroup, MetricGroup};
use measured_prometheus_protobuf::ProtoEncoder;
use prometheus::Encoder;
use prometheus_client::encoding::EncodeLabelSet;

fn main() {
    divan::Divan::from_args()
        .threads([1])
        .sample_size(10)
        .sample_count(100)
        .run_benches();
}

const SIZES: &[usize] = &[100, 1000, 10000, 100000];

#[divan::bench(consts = SIZES)]
fn measured<const N: usize>(bencher: Bencher) {
    let metrics = Metrics {
        counters: measured::CounterVec::with_label_set(GroupSet {
            kind: ThreadedRodeo::with_hasher(ahash::RandomState::new()),
        }),
    };

    let mut buf = itoa::Buffer::new();
    for i in 0..N {
        metrics.counters.inc(Group {
            kind: buf.format(i),
        });
    }

    let mut enc = ProtoEncoder::new(vec![]);

    bencher.bench_local(|| {
        enc.writer.clear();
        metrics.collect_group_into(&mut enc).unwrap();
        enc.flush().unwrap();
        black_box(&enc.writer);
    });
}

#[divan::bench(consts = SIZES)]
fn prometheus<const N: usize>(bencher: Bencher) {
    let registry = prometheus::Registry::new();
    let counter_vec = prometheus::register_int_counter_vec_with_registry!(
        "counters",
        "help text",
        &["kind"],
        registry
    )
    .unwrap();

    let mut buf = itoa::Buffer::new();
    for i in 0..N {
        counter_vec.with_label_values(&[buf.format(i)]).inc();
    }

    let mut enc = Vec::new();

    bencher.bench_local(|| {
        enc.clear();
        prometheus::ProtobufEncoder::new()
            .encode(&registry.gather(), &mut enc)
            .unwrap();
        black_box(&enc);
    });
}

#[divan::bench(consts = SIZES)]
fn prometheus_client<const N: usize>(bencher: Bencher) {
    use prometheus_client::metrics::counter::Counter;
    use prometheus_client::metrics::family::Family;
    use prometheus_client::registry::Registry;
    use prost12::Message;

    let mut registry = <Registry>::default();

    let counter_vec = Family::<GroupStatic, Counter>::default();

    registry.register("counters", "help text", counter_vec.clone());

    let mut buf = itoa::Buffer::new();
    for i in 0..N {
        let kind = buf.format(i).to_owned();
        counter_vec.get_or_create(&GroupStatic { kind }).inc();
    }

    let mut enc = Vec::new();

    bencher.bench_local(|| {
        enc.clear();
        prometheus_client::encoding::protobuf::encode(&registry)
            .unwrap()
            .encode(&mut enc)
            .unwrap();
        black_box(&enc);
    });
}

#[derive(MetricGroup)]
struct Metrics {
    /// help text
    counters: CounterVec<GroupSet>,
}

#[derive(Clone, Copy, PartialEq, Debug, LabelGroup)]
#[label(set = GroupSet)]
struct Group<'a> {
    #[label(dynamic_with = ThreadedRodeo<Spur, ahash::RandomState>)]
    kind: &'a str,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct GroupStatic {
    kind: String,
}
