#![allow(clippy::cast_precision_loss)]

use std::io::Write;

use encoding::{encode_key, encode_varint, encoded_len_varint, key_len, WireType::LengthDelimited};
use measured::{
    label::{LabelGroupVisitor, LabelName, LabelValue, LabelVisitor},
    metric::{
        counter::CounterState,
        gauge::{FloatGaugeState, GaugeState},
        group::Encoding,
        name::MetricNameEncoder,
        MetricEncoding,
    },
    LabelGroup,
};

mod encoding;

/// The prometheus text encoder helper
pub struct ProtoEncoder<W> {
    state: State,
    pub writer: W,
    buf: Vec<u8>,
}

impl<W: Write> ProtoEncoder<W> {
    /// Create a new text encoder.
    ///
    /// This should ideally be cached and re-used between collections to reduce re-allocating
    pub fn new(w: W) -> Self {
        Self {
            state: State::Init,
            writer: w,
            buf: Vec::new(),
        }
    }

    /// Finish the text encoding and extract the bytes to send in a HTTP response.
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.flush_buf()?;
        self.writer.flush()
    }

    fn flush_buf(&mut self) -> Result<(), std::io::Error> {
        if self.state == State::Metrics {
            self.state = State::Init;

            let len = self.buf.len() - 10;
            let varint_len = encoded_len_varint(len as u64);
            let offset = 10 - varint_len;
            encode_varint(len as u64, &mut &mut self.buf[offset..]);
            self.writer.write_all(&self.buf[offset..])?;

            self.buf.resize(10, 0);
        } else if self.buf.is_empty() {
            self.buf.resize(10, 0);
        }
        debug_assert!(self.buf.len() >= 10);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Init,
    Help,
    Metrics,
}

/// Prometheus only supports these 5 types of metrics
#[derive(Clone, Copy, Debug)]
pub enum MetricType {
    /// Corresponds to [`Counter`](crate::Counter)
    Counter,
    /// Corresponds to [`Histogram`](crate::Histogram)
    Histogram,
    /// Corresponds to [`Gauge`](crate::Gauge)
    Gauge,
    /// Not currently supported
    Summary,
    /// Not currently supported
    Untyped,
}

impl<W: Write> Encoding for ProtoEncoder<W> {
    type Err = std::io::Error;

    /// Write the help line for a metric
    fn write_help(
        &mut self,
        name: impl MetricNameEncoder,
        help: &str,
    ) -> Result<(), std::io::Error> {
        self.flush_buf()?;

        // optional string     name   = 1;
        encode_key(1, LengthDelimited, &mut self.buf);
        encode_varint(name.encode_len() as u64, &mut self.buf);
        name.encode_utf8(&mut self.buf)?;

        // optional string     help   = 2;
        encoding::encode_str(2, help, &mut self.buf);

        self.state = State::Help;

        Ok(())
    }
}

struct LenVisitor {}
impl LabelVisitor for LenVisitor {
    type Output = usize;
    fn write_int(self, x: i64) -> usize {
        encoding::encoded_len_str(2, itoa::Buffer::new().format(x))
    }

    fn write_float(self, x: f64) -> usize {
        if x.is_infinite() {
            if x.is_sign_positive() {
                encoding::encoded_len_str(2, "+Inf")
            } else {
                encoding::encoded_len_str(2, "-Inf")
            }
        } else if x.is_nan() {
            encoding::encoded_len_str(2, "NaN")
        } else {
            encoding::encoded_len_str(2, ryu::Buffer::new().format(x))
        }
    }

    fn write_str(self, x: &str) -> usize {
        encoding::encoded_len_str(2, x)
    }
}

struct GroupLenVisitor {
    len: usize,
}
impl LabelGroupVisitor for GroupLenVisitor {
    type Output = ();
    fn write_value(&mut self, name: &LabelName, x: &impl LabelValue) {
        let mut label_pair_len = 0;

        label_pair_len += encoding::encoded_len_str(1, name.as_str());
        label_pair_len += x.visit(LenVisitor {});

        self.len += message_len(1, label_pair_len);
    }
}

struct Visitor<'a> {
    buf: &'a mut Vec<u8>,
}
impl LabelVisitor for Visitor<'_> {
    type Output = ();
    fn write_int(self, x: i64) {
        self.write_str(itoa::Buffer::new().format(x));
    }

    fn write_float(self, x: f64) {
        if x.is_infinite() {
            if x.is_sign_positive() {
                self.write_str("+Inf");
            } else {
                self.write_str("-Inf");
            }
        } else if x.is_nan() {
            self.write_str("NaN");
        } else {
            self.write_str(ryu::Buffer::new().format(x));
        }
    }

    fn write_str(self, x: &str) {
        // optional string value = 2;
        encoding::encode_str(2, x, self.buf);
    }
}

fn encode_message(
    tag: u32,
    len: usize,
    buf: &mut Vec<u8>,
    message: impl for<'a> FnOnce(&'a mut Vec<u8>),
) {
    encode_key(tag, LengthDelimited, buf);
    encode_varint(len as u64, buf);

    {
        let offset = buf.len();
        message(buf);
        debug_assert_eq!(buf.len() - offset, len);
    }
}

fn message_len(tag: u32, len: usize) -> usize {
    key_len(tag) + encoded_len_varint(len as u64) + len
}

struct GroupVisitor<'a> {
    buf: &'a mut Vec<u8>,
}
impl LabelGroupVisitor for GroupVisitor<'_> {
    type Output = ();
    fn write_value(&mut self, name: &LabelName, x: &impl LabelValue) {
        let mut label_pair_len = 0;
        label_pair_len += encoding::encoded_len_str(1, name.as_str());
        label_pair_len += x.visit(LenVisitor {});

        // repeated LabelPair label        = 1;
        encode_message(1, label_pair_len, self.buf, |buf| {
            // optional string name  = 1;
            encoding::encode_str(1, name.as_str(), buf);

            x.visit(Visitor { buf });
        });
    }
}

impl<W: Write> MetricEncoding<ProtoEncoder<W>> for CounterState {
    fn write_type(
        name: impl MetricNameEncoder,
        enc: &mut ProtoEncoder<W>,
    ) -> Result<(), std::io::Error> {
        enc.flush_buf()?;

        if enc.state == State::Init {
            // optional string     name   = 1;
            encode_key(1, LengthDelimited, &mut enc.buf);
            encode_varint(name.encode_len() as u64, &mut enc.buf);
            name.encode_utf8(&mut enc.buf)?;
        }

        // optional MetricType type   = 3;
        // COUNTER = 0;
        encoding::encode_i32(3, 0, &mut enc.buf);

        Ok(())
    }

    fn collect_into(
        &self,
        _m: &(),
        labels: impl LabelGroup,
        _name: impl MetricNameEncoder,
        enc: &mut ProtoEncoder<W>,
    ) -> Result<(), std::io::Error> {
        enc.state = State::Metrics;

        let mut metric_len = 0;

        let mut label_pairs_len = GroupLenVisitor { len: 0 };
        labels.visit_values(&mut label_pairs_len);
        metric_len += label_pairs_len.len;

        let count = self.count.load(std::sync::atomic::Ordering::Relaxed) as f64;
        let count_len = encoding::encoded_len_f64(1, count);
        metric_len += message_len(3, count_len);

        // repeated Metric     metric = 4;
        encode_message(4, metric_len, &mut enc.buf, |buf| {
            labels.visit_values(&mut GroupVisitor { buf });

            // optional Counter   counter      = 3;
            encode_message(3, count_len, buf, |buf| {
                // optional double   value    = 1;
                encoding::encode_f64(1, count, buf);
            });
        });

        Ok(())
    }
}

impl<W: Write> MetricEncoding<ProtoEncoder<W>> for GaugeState {
    fn write_type(
        name: impl MetricNameEncoder,
        enc: &mut ProtoEncoder<W>,
    ) -> Result<(), std::io::Error> {
        enc.flush_buf()?;

        if enc.state == State::Init {
            // optional string     name   = 1;
            encode_key(1, LengthDelimited, &mut enc.buf);
            encode_varint(name.encode_len() as u64, &mut enc.buf);
            name.encode_utf8(&mut enc.buf)?;
        }

        // optional MetricType type   = 3;
        // GAUGE = 1;
        encoding::encode_i32(3, 1, &mut enc.buf);

        Ok(())
    }

    fn collect_into(
        &self,
        _m: &(),
        labels: impl LabelGroup,
        _name: impl MetricNameEncoder,
        enc: &mut ProtoEncoder<W>,
    ) -> Result<(), std::io::Error> {
        enc.state = State::Metrics;

        let mut metric_len = 0;

        let mut label_pairs_len = GroupLenVisitor { len: 0 };
        labels.visit_values(&mut label_pairs_len);
        metric_len += label_pairs_len.len;

        let gauge = self.count.load(std::sync::atomic::Ordering::Relaxed) as f64;
        let gauge_len = encoding::encoded_len_f64(1, gauge);
        metric_len += message_len(3, gauge_len);

        // repeated Metric     metric = 4;
        encode_message(4, metric_len, &mut enc.buf, |buf| {
            labels.visit_values(&mut GroupVisitor { buf });

            // optional Gauge   gauge      = 2;
            encode_message(2, gauge_len, buf, |buf| {
                // optional double   value    = 1;
                encoding::encode_f64(1, gauge, buf);
            });
        });

        Ok(())
    }
}

impl<W: Write> MetricEncoding<ProtoEncoder<W>> for FloatGaugeState {
    fn write_type(
        name: impl MetricNameEncoder,
        enc: &mut ProtoEncoder<W>,
    ) -> Result<(), std::io::Error> {
        enc.flush_buf()?;

        if enc.state == State::Init {
            // optional string     name   = 1;
            encode_key(1, LengthDelimited, &mut enc.buf);
            encode_varint(name.encode_len() as u64, &mut enc.buf);
            name.encode_utf8(&mut enc.buf)?;
        }

        // optional MetricType type   = 3;
        // GAUGE = 1;
        encoding::encode_i32(3, 1, &mut enc.buf);

        Ok(())
    }

    fn collect_into(
        &self,
        _m: &(),
        labels: impl LabelGroup,
        _name: impl MetricNameEncoder,
        enc: &mut ProtoEncoder<W>,
    ) -> Result<(), std::io::Error> {
        enc.state = State::Metrics;

        let mut metric_len = 0;

        let mut label_pairs_len = GroupLenVisitor { len: 0 };
        labels.visit_values(&mut label_pairs_len);
        metric_len += label_pairs_len.len;

        let gauge = self.count.get();
        let gauge_len = encoding::encoded_len_f64(1, gauge);
        metric_len += message_len(3, gauge_len);

        // repeated Metric     metric = 4;
        encode_message(4, metric_len, &mut enc.buf, |buf| {
            labels.visit_values(&mut GroupVisitor { buf });

            // optional Gauge   gauge      = 2;
            encode_message(2, gauge_len, buf, |buf| {
                // optional double   value    = 1;
                encoding::encode_f64(1, gauge, buf);
            });
        });

        Ok(())
    }
}

#[cfg(test)]
mod generated;

#[cfg(test)]
mod tests {
    use std::vec;

    use bytes::{BufMut, BytesMut};
    use measured::{
        metric::{
            group::Encoding,
            name::{MetricName, Total},
            MetricFamilyEncoding,
        },
        CounterVec, GaugeVec,
    };
    use prost::Message;

    use crate::{
        generated::{Counter, Gauge, LabelPair, Metric, MetricFamily, MetricType},
        ProtoEncoder,
    };

    #[derive(Clone, Copy, PartialEq, Debug, measured::LabelGroup)]
    #[label(set = RequestLabelSet)]
    struct RequestLabels {
        method: Method,
        code: StatusCode,
    }

    #[derive(Clone, Copy, PartialEq, Debug, measured::FixedCardinalityLabel)]
    #[label(rename_all = "snake_case")]
    enum Method {
        Post,
        Get,
    }

    #[derive(Clone, Copy, PartialEq, Debug, measured::FixedCardinalityLabel)]
    enum StatusCode {
        Ok = 200,
        BadRequest = 400,
    }

    #[test]
    fn counters() {
        let requests = CounterVec::<RequestLabelSet>::new();

        let labels = RequestLabels {
            method: Method::Post,
            code: StatusCode::Ok,
        };
        requests.inc_by(labels, 1027);

        let labels = RequestLabels {
            method: Method::Get,
            code: StatusCode::BadRequest,
        };
        requests.inc_by(labels, 3);

        let mut enc = ProtoEncoder::new(BytesMut::new().writer());

        let name = MetricName::from_str("http_request").with_suffix(Total);
        enc.write_help(&name, "The total number of HTTP requests.")
            .unwrap();
        requests.collect_family_into(&name, &mut enc).unwrap();
        enc.flush().unwrap();
        let actual_msg = enc.writer.into_inner();

        let expected = MetricFamily {
            name: Some("http_request_total".to_string()),
            help: Some("The total number of HTTP requests.".to_string()),
            r#type: Some(MetricType::Counter as i32),
            metric: vec![
                Metric {
                    label: vec![
                        LabelPair {
                            name: Some("method".to_owned()),
                            value: Some("post".to_owned()),
                        },
                        LabelPair {
                            name: Some("code".to_owned()),
                            value: Some("200".to_owned()),
                        },
                    ],
                    gauge: None,
                    counter: Some(Counter {
                        value: Some(1027.0),
                        exemplar: None,
                        created_timestamp: None,
                    }),
                    summary: None,
                    untyped: None,
                    histogram: None,
                    timestamp_ms: None,
                },
                Metric {
                    label: vec![
                        LabelPair {
                            name: Some("method".to_owned()),
                            value: Some("get".to_owned()),
                        },
                        LabelPair {
                            name: Some("code".to_owned()),
                            value: Some("400".to_owned()),
                        },
                    ],
                    gauge: None,
                    counter: Some(Counter {
                        value: Some(3.0),
                        exemplar: None,
                        created_timestamp: None,
                    }),
                    summary: None,
                    untyped: None,
                    histogram: None,
                    timestamp_ms: None,
                },
            ],
            unit: None,
        };
        let mut expected_msg = BytesMut::new();
        expected.encode_length_delimited(&mut expected_msg).unwrap();

        assert_eq!(actual_msg, expected_msg);

        let actual = MetricFamily::decode_length_delimited(actual_msg).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn gauge() {
        let requests = GaugeVec::<RequestLabelSet>::new();

        let labels = RequestLabels {
            method: Method::Post,
            code: StatusCode::Ok,
        };
        requests.inc_by(labels, 1027);

        let labels = RequestLabels {
            method: Method::Get,
            code: StatusCode::BadRequest,
        };
        requests.inc_by(labels, 3);

        let mut enc = ProtoEncoder::new(BytesMut::new().writer());

        let name = MetricName::from_str("http_request").with_suffix(Total);
        enc.write_help(&name, "The total number of HTTP requests.")
            .unwrap();
        requests.collect_family_into(&name, &mut enc).unwrap();
        enc.flush().unwrap();
        let actual_msg = enc.writer.into_inner();

        let expected = MetricFamily {
            name: Some("http_request_total".to_string()),
            help: Some("The total number of HTTP requests.".to_string()),
            r#type: Some(MetricType::Gauge as i32),
            metric: vec![
                Metric {
                    label: vec![
                        LabelPair {
                            name: Some("method".to_owned()),
                            value: Some("post".to_owned()),
                        },
                        LabelPair {
                            name: Some("code".to_owned()),
                            value: Some("200".to_owned()),
                        },
                    ],
                    gauge: Some(Gauge {
                        value: Some(1027.0),
                    }),
                    counter: None,
                    summary: None,
                    untyped: None,
                    histogram: None,
                    timestamp_ms: None,
                },
                Metric {
                    label: vec![
                        LabelPair {
                            name: Some("method".to_owned()),
                            value: Some("get".to_owned()),
                        },
                        LabelPair {
                            name: Some("code".to_owned()),
                            value: Some("400".to_owned()),
                        },
                    ],
                    gauge: Some(Gauge { value: Some(3.0) }),
                    counter: None,
                    summary: None,
                    untyped: None,
                    histogram: None,
                    timestamp_ms: None,
                },
            ],
            unit: None,
        };
        let mut expected_msg = BytesMut::new();
        expected.encode_length_delimited(&mut expected_msg).unwrap();

        assert_eq!(actual_msg, expected_msg);

        let actual = MetricFamily::decode_length_delimited(actual_msg).unwrap();
        assert_eq!(actual, expected);
    }
}
