use std::io::Write;

use encoding::{encode_key, encode_varint, encoded_len_varint, key_len, WireType::LengthDelimited};
use measured::{
    label::{LabelGroupVisitor, LabelName, LabelValue, LabelVisitor},
    metric::{
        counter::CounterState,
        group::{Encoding, MetricValue},
        name::MetricNameEncoder,
        MetricEncoding,
    },
    LabelGroup,
};

mod encoding;

// impl ::prost::Message for MetricFamily {
//     #[allow(unused_variables)]
//     fn encode_raw<B>(&self, buf: &mut B)
//     where
//         B: ::prost::bytes::BufMut,
//     {
//         if let ::core::option::Option::Some(ref value) = self.name {
//             ::prost::encoding::string::encode(1u32, value, buf);
//         }
//         if let ::core::option::Option::Some(ref value) = self.help {
//             ::prost::encoding::string::encode(2u32, value, buf);
//         }
//         if let ::core::option::Option::Some(ref value) = self.r#type {
//             ::prost::encoding::int32::encode(3u32, value, buf);
//         }
//         for msg in &self.metric {
//             ::prost::encoding::message::encode(4u32, msg, buf);
//         }
//         if let ::core::option::Option::Some(ref value) = self.unit {
//             ::prost::encoding::string::encode(5u32, value, buf);
//         }
//     }
// }
// impl ::prost::Message for Metric {
//     #[allow(unused_variables)]
//     fn encode_raw<B>(&self, buf: &mut B)
//     where
//         B: ::prost::bytes::BufMut,
//     {
//         for msg in &self.label {
//             ::prost::encoding::message::encode(1u32, msg, buf);
//         }
//         if let Some(ref msg) = self.gauge {
//             ::prost::encoding::message::encode(2u32, msg, buf);
//         }
//         if let Some(ref msg) = self.counter {
//             ::prost::encoding::message::encode(3u32, msg, buf);
//         }
//         if let Some(ref msg) = self.summary {
//             ::prost::encoding::message::encode(4u32, msg, buf);
//         }
//         if let Some(ref msg) = self.untyped {
//             ::prost::encoding::message::encode(5u32, msg, buf);
//         }
//         if let ::core::option::Option::Some(ref value) = self.timestamp_ms {
//             ::prost::encoding::int64::encode(6u32, value, buf);
//         }
//         if let Some(ref msg) = self.histogram {
//             ::prost::encoding::message::encode(7u32, msg, buf);
//         }
//     }
// }

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
        encoding::string::encode(2, help, &mut self.buf);

        self.state = State::Help;

        Ok(())
    }

    /// Write the metric data
    fn write_metric_value(
        &mut self,
        _name: impl MetricNameEncoder,
        _labels: impl LabelGroup,
        _value: MetricValue,
    ) -> Result<(), std::io::Error> {
        // struct Visitor<'a, W> {
        //     writer: &'a mut W,
        // }
        // impl<W: Write> LabelVisitor for Visitor<'_, W> {
        //     type Output = Result<(), std::io::Error>;
        //     fn write_int(self, x: i64) -> Result<(), std::io::Error> {
        //         self.write_str(itoa::Buffer::new().format(x))
        //     }

        //     fn write_float(self, x: f64) -> Result<(), std::io::Error> {
        //         if x.is_infinite() {
        //             if x.is_sign_positive() {
        //                 self.write_str("+Inf")
        //             } else {
        //                 self.write_str("-Inf")
        //             }
        //         } else if x.is_nan() {
        //             self.write_str("NaN")
        //         } else {
        //             self.write_str(ryu::Buffer::new().format(x))
        //         }
        //     }

        //     fn write_str(self, x: &str) -> Result<(), std::io::Error> {
        //         self.writer.write_all(b"=\"")?;
        //         // write_label_str_value(x, &mut *self.writer)?;
        //         self.writer.write_all(b"\"")?;
        //         Ok(())
        //     }
        // }

        // struct GroupVisitor<'a, W> {
        //     first: bool,
        //     writer: &'a mut W,
        // }
        // impl<W: Write> LabelGroupVisitor for GroupVisitor<'_, W> {
        //     type Output = Result<(), std::io::Error>;
        //     fn write_value(
        //         &mut self,
        //         name: &LabelName,
        //         x: &impl LabelValue,
        //     ) -> Result<(), std::io::Error> {
        //         if self.first {
        //             self.first = false;
        //             self.writer.write_all(b"{")?;
        //         } else {
        //             self.writer.write_all(b",")?;
        //         }
        //         self.writer.write_all(name.as_str().as_bytes())?;
        //         x.visit(Visitor {
        //             writer: self.writer,
        //         })
        //     }
        // }

        // self.state = State::Metrics;
        // name.encode_utf8(&mut self.writer)?;

        // let mut visitor = GroupVisitor {
        //     first: true,
        //     writer: &mut self.writer,
        // };
        // labels.visit_values(&mut visitor);
        // if !visitor.first {
        //     self.writer.write_all(b"}")?;
        // }
        // self.writer.write_all(b" ")?;
        // match value {
        //     MetricValue::Int(x) => self
        //         .writer
        //         .write_all(itoa::Buffer::new().format(x).as_bytes())?,
        //     MetricValue::Float(x) => self
        //         .writer
        //         .write_all(ryu::Buffer::new().format(x).as_bytes())?,
        // }
        // self.writer.write_all(b"\n")?;
        Ok(())
    }
}

struct LenVisitor {}
impl LabelVisitor for LenVisitor {
    type Output = usize;
    fn write_int(self, x: i64) -> usize {
        encoding::string::encoded_len(2, itoa::Buffer::new().format(x))
    }

    fn write_float(self, x: f64) -> usize {
        if x.is_infinite() {
            if x.is_sign_positive() {
                encoding::string::encoded_len(2, "+Inf")
            } else {
                encoding::string::encoded_len(2, "-Inf")
            }
        } else if x.is_nan() {
            encoding::string::encoded_len(2, "NaN")
        } else {
            encoding::string::encoded_len(2, ryu::Buffer::new().format(x))
        }
    }

    fn write_str(self, x: &str) -> usize {
        encoding::string::encoded_len(2, x)
    }
}

struct GroupLenVisitor {
    len: usize,
}
impl LabelGroupVisitor for GroupLenVisitor {
    type Output = ();
    fn write_value(&mut self, name: &LabelName, x: &impl LabelValue) {
        let mut label_pair_len = 0;

        label_pair_len += encoding::string::encoded_len(1, name.as_str());
        label_pair_len += x.visit(LenVisitor {});

        self.len += key_len(1);
        self.len += encoded_len_varint(label_pair_len as u64);
        self.len += label_pair_len;
    }
}

struct Visitor<'a> {
    buf: &'a mut Vec<u8>,
}
impl LabelVisitor for Visitor<'_> {
    type Output = ();
    fn write_int(self, x: i64) {
        self.write_str(itoa::Buffer::new().format(x))
    }

    fn write_float(self, x: f64) {
        if x.is_infinite() {
            if x.is_sign_positive() {
                self.write_str("+Inf")
            } else {
                self.write_str("-Inf")
            }
        } else if x.is_nan() {
            self.write_str("NaN")
        } else {
            self.write_str(ryu::Buffer::new().format(x))
        }
    }

    fn write_str(self, x: &str) {
        // optional string value = 2;
        encoding::string::encode(2, x, self.buf);
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

struct GroupVisitor<'a> {
    buf: &'a mut Vec<u8>,
}
impl LabelGroupVisitor for GroupVisitor<'_> {
    type Output = ();
    fn write_value(&mut self, name: &LabelName, x: &impl LabelValue) {
        let mut label_pair_len = 0;
        label_pair_len += encoding::string::encoded_len(1, name.as_str());
        label_pair_len += x.visit(LenVisitor {});

        // repeated LabelPair label        = 1;
        encode_message(1, label_pair_len, self.buf, |buf| {
            // optional string name  = 1;
            encoding::string::encode(1, name.as_str(), buf);

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
        encoding::int32::encode(3, &0, &mut enc.buf);

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
        let count_len = encoding::double::encoded_len(1, &count);
        metric_len += key_len(3) + encoded_len_varint(count_len as u64) + count_len;

        // repeated Metric     metric = 4;
        encode_message(4, metric_len, &mut enc.buf, |buf| {
            labels.visit_values(&mut GroupVisitor { buf });

            // optional Counter   counter      = 3;
            encode_key(3, LengthDelimited, buf);
            encode_varint(count_len as u64, buf);
            encoding::double::encode(1, &count, buf);
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
        CounterVec,
    };
    use prost::Message;

    use crate::{
        generated::{Counter, LabelPair, Metric, MetricFamily, MetricType},
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
}

// let s = String::from_utf8(encoder.finish().to_vec()).unwrap();
// assert_eq!(
//     s,
//     r#"# HELP http_request_total The total number of HTTP requests.
// # TYPE http_request_total counter
// http_request_total{method="post",code="200"} 1027
// http_request_total{method="get",code="400"} 3
// "#

// b"\x8d\x01\n\x12http_request_total\x12\"The total number of HTTP requests.\x18\0\"(\n\x0e\n\x06method\x12\x04post\n\x0b\n\x04code\x12\x03200\x1a\t\t\0\0\0\0\0\x0c\x90@\"'\n\r\n\x06method\x12\x03get\n\x0b\n\x04code\x12\x03400\x1a\t\t\0\0\0\0\0\0\x08@"
// b"\x8d\x01\n\x12http_request_total\x12\"The total number of HTTP requests.\x10\0\"(\n\x0e\n\x06method\x12\x04post\n\x0b\n\x04code\x12\x03200\x1a\t\t\0\0\0\0\0\x0c\x90@\"'\n\r\n\x06method\x12\x03get\n\x0b\n\x04code\x12\x03400\x1a\t\t\0\0\0\0\0\0\x08@"
