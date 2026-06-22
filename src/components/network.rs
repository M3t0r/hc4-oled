use super::{Component, Drawer, Error};

use embedded_graphics::{
    prelude::*,
    primitives::Line,
    text::{Baseline, Text},
};

use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

const ARROW_WIDTH: i32 = 3;
const ELEMENT_PADDING: i32 = 1;
const FONT_WIDTH: i32 = 6;
const RATE_WIDTH: i32 = 4 * FONT_WIDTH;
const GROUP_WIDTH: i32 = ARROW_WIDTH + ELEMENT_PADDING + RATE_WIDTH;
const GROUP_GAP: i32 = Drawer::WIDTH as i32 - (2 * GROUP_WIDTH);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NetworkCounters {
    rx_bytes: u64,
    tx_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NetworkRates {
    rx_bytes_per_second: u64,
    tx_bytes_per_second: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NetworkSample {
    counters: NetworkCounters,
    measured_at: Instant,
}

#[derive(Debug)]
pub struct NetworkThroughput {
    name: String,
    interface_path: PathBuf,
    previous_sample: Option<NetworkSample>,
    rates: Option<NetworkRates>,
}

impl NetworkThroughput {
    pub fn new(name: String, sysfs_root: &Path) -> Result<Self, Error> {
        let interface_path = find_interface_path(sysfs_root, &name)?;

        Ok(Self {
            name,
            interface_path,
            previous_sample: None,
            rates: None,
        })
    }

    fn read_counters(&self) -> Result<NetworkCounters, Error> {
        read_counters(&self.interface_path)
    }
}

impl std::fmt::Display for NetworkThroughput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Network {}", self.name)
    }
}

impl Component for NetworkThroughput {
    fn should_update(&self, last_update: std::time::Duration) -> bool {
        last_update > std::time::Duration::from_secs(5)
    }

    fn update(&mut self) -> Result<(), Error> {
        let sample = NetworkSample {
            counters: self.read_counters()?,
            measured_at: Instant::now(),
        };

        self.rates = self
            .previous_sample
            .and_then(|previous_sample| calculate_rates(previous_sample, sample));
        self.previous_sample = Some(sample);

        Ok(())
    }

    fn draw(&self, drawable: &mut Drawer, offset: Point, _tick: u64) -> Result<(), Error> {
        let rx_offset = offset;
        let tx_offset = offset + Point::new(GROUP_WIDTH + GROUP_GAP, 0);
        let rx_rate = format_rate(self.rates.map(|rates| rates.rx_bytes_per_second));
        let tx_rate = format_rate(self.rates.map(|rates| rates.tx_bytes_per_second));

        Text::with_baseline(
            &rx_rate,
            rx_offset + Point::new(RATE_WIDTH - rx_rate.len() as i32 * FONT_WIDTH, 0),
            drawable.base_text_style,
            Baseline::Top,
        )
        .draw(&mut drawable.display)?;
        draw_down_arrow(
            drawable,
            rx_offset + Point::new(RATE_WIDTH + ELEMENT_PADDING, 2),
        )?;

        Text::with_baseline(
            &tx_rate,
            tx_offset + Point::new(RATE_WIDTH - tx_rate.len() as i32 * FONT_WIDTH, 0),
            drawable.base_text_style,
            Baseline::Top,
        )
        .draw(&mut drawable.display)?;
        draw_up_arrow(
            drawable,
            tx_offset + Point::new(RATE_WIDTH + ELEMENT_PADDING, 2),
        )?;

        Ok(())
    }
}

fn find_interface_path(sysfs_root: &Path, name: &str) -> Result<PathBuf, Error> {
    let name = name.trim_end_matches(':');

    if name.is_empty() {
        return Err("network adapter name must not be empty".into());
    }

    if name.contains('/') {
        return Err("network adapter name must not contain '/'".into());
    }

    let candidates = candidate_interface_names(name);
    for candidate in candidates {
        let path = sysfs_root.join(candidate);
        if path.is_dir() {
            return Ok(path);
        }
    }

    Err(format!(
        "Could not find network adapter '{}' in {}",
        name,
        sysfs_root.display()
    )
    .into())
}

fn candidate_interface_names(name: &str) -> Vec<&str> {
    if let Some((base_name, _)) = name.split_once('@') {
        if !base_name.is_empty() {
            return vec![name, base_name];
        }
    }

    vec![name]
}

fn read_counters(interface_path: &Path) -> Result<NetworkCounters, Error> {
    let statistics_path = interface_path.join("statistics");
    Ok(NetworkCounters {
        rx_bytes: read_counter(&statistics_path.join("rx_bytes"))?,
        tx_bytes: read_counter(&statistics_path.join("tx_bytes"))?,
    })
}

fn read_counter(path: &Path) -> Result<u64, Error> {
    Ok(fs::read_to_string(path)?
        .trim()
        .parse()
        .map_err(|_| format!("Could not parse network counter {}", path.display()))?)
}

fn calculate_rates(previous_sample: NetworkSample, sample: NetworkSample) -> Option<NetworkRates> {
    let elapsed = sample
        .measured_at
        .checked_duration_since(previous_sample.measured_at)?;

    if elapsed.is_zero() {
        return None;
    }

    Some(NetworkRates {
        rx_bytes_per_second: bytes_per_second(
            sample
                .counters
                .rx_bytes
                .saturating_sub(previous_sample.counters.rx_bytes),
            elapsed,
        ),
        tx_bytes_per_second: bytes_per_second(
            sample
                .counters
                .tx_bytes
                .saturating_sub(previous_sample.counters.tx_bytes),
            elapsed,
        ),
    })
}

fn bytes_per_second(bytes: u64, elapsed: Duration) -> u64 {
    ((bytes as u128 * 1_000_000_000) / elapsed.as_nanos()) as u64
}

fn format_rate(bytes_per_second: Option<u64>) -> String {
    let Some(bytes_per_second) = bytes_per_second else {
        return "-".to_string();
    };

    const UNITS: [&str; 7] = ["B", "K", "M", "G", "T", "P", "E"];
    let mut value = bytes_per_second;
    let mut magnitude = 0usize;

    while value >= 1000 && magnitude + 1 < UNITS.len() {
        let previous_value = value;
        value /= 1000;
        magnitude += 1;

        if previous_value - (value * 1000) > 900 {
            value += 1;
        }
    }

    format!("{}{}", value, UNITS[magnitude])
}

fn draw_down_arrow(drawable: &mut Drawer, offset: Point) -> Result<(), Error> {
    Line::new(offset + Point::new(1, 0), offset + Point::new(1, 4))
        .into_styled(drawable.base_primitive_style)
        .draw(&mut drawable.display)?;
    Line::new(offset + Point::new(0, 3), offset + Point::new(1, 4))
        .into_styled(drawable.base_primitive_style)
        .draw(&mut drawable.display)?;
    Line::new(offset + Point::new(2, 3), offset + Point::new(1, 4))
        .into_styled(drawable.base_primitive_style)
        .draw(&mut drawable.display)?;

    Ok(())
}

fn draw_up_arrow(drawable: &mut Drawer, offset: Point) -> Result<(), Error> {
    Line::new(offset + Point::new(1, 0), offset + Point::new(1, 4))
        .into_styled(drawable.base_primitive_style)
        .draw(&mut drawable.display)?;
    Line::new(offset + Point::new(0, 1), offset + Point::new(1, 0))
        .into_styled(drawable.base_primitive_style)
        .draw(&mut drawable.display)?;
    Line::new(offset + Point::new(2, 1), offset + Point::new(1, 0))
        .into_styled(drawable.base_primitive_style)
        .draw(&mut drawable.display)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Write;

    #[test]
    fn candidate_names_include_ip_link_peer_suffix_base() {
        assert_eq!(
            candidate_interface_names("eth0@if3"),
            vec!["eth0@if3", "eth0"]
        );
    }

    #[test]
    fn find_interface_path_accepts_ip_link_trailing_colon() {
        let root = tempfile_dir("colon");
        fs::create_dir(root.join("eth0")).unwrap();

        assert_eq!(
            find_interface_path(&root, "eth0@if3:").unwrap(),
            root.join("eth0")
        );
    }

    #[test]
    fn rates_are_none_for_initial_sample() {
        let now = Instant::now();
        let sample = NetworkSample {
            counters: NetworkCounters {
                rx_bytes: 100,
                tx_bytes: 200,
            },
            measured_at: now,
        };

        assert_eq!(calculate_rates(sample, sample), None);
    }

    #[test]
    fn rates_are_calculated_from_rx_and_tx_deltas() {
        let now = Instant::now();
        let previous_sample = NetworkSample {
            counters: NetworkCounters {
                rx_bytes: 100,
                tx_bytes: 200,
            },
            measured_at: now,
        };
        let sample = NetworkSample {
            counters: NetworkCounters {
                rx_bytes: 350,
                tx_bytes: 700,
            },
            measured_at: now + Duration::from_secs(5),
        };

        assert_eq!(
            calculate_rates(previous_sample, sample),
            Some(NetworkRates {
                rx_bytes_per_second: 50,
                tx_bytes_per_second: 100,
            })
        );
    }

    #[test]
    fn rates_saturate_when_counters_decrease() {
        let now = Instant::now();
        let previous_sample = NetworkSample {
            counters: NetworkCounters {
                rx_bytes: 100,
                tx_bytes: 200,
            },
            measured_at: now,
        };
        let sample = NetworkSample {
            counters: NetworkCounters {
                rx_bytes: 50,
                tx_bytes: 100,
            },
            measured_at: now + Duration::from_secs(5),
        };

        assert_eq!(
            calculate_rates(previous_sample, sample),
            Some(NetworkRates {
                rx_bytes_per_second: 0,
                tx_bytes_per_second: 0,
            })
        );
    }

    #[test]
    fn rates_use_actual_elapsed_time() {
        let now = Instant::now();
        let previous_sample = NetworkSample {
            counters: NetworkCounters {
                rx_bytes: 0,
                tx_bytes: 0,
            },
            measured_at: now,
        };
        let sample = NetworkSample {
            counters: NetworkCounters {
                rx_bytes: 100,
                tx_bytes: 50,
            },
            measured_at: now + Duration::from_secs(2),
        };

        assert_eq!(
            calculate_rates(previous_sample, sample),
            Some(NetworkRates {
                rx_bytes_per_second: 50,
                tx_bytes_per_second: 25,
            })
        );
    }

    #[test]
    fn format_rate_uses_placeholder_before_calculation() {
        assert_eq!(format_rate(None), "-.-");
    }

    #[test]
    fn format_rate_uses_metric_byte_units() {
        assert_eq!(format_rate(Some(0)), "0B");
        assert_eq!(format_rate(Some(999)), "999B");
        assert_eq!(format_rate(Some(1_500)), "1K");
        assert_eq!(format_rate(Some(1_500_000)), "1M");
        assert_eq!(format_rate(Some(999_999)), "1M");
        assert_eq!(format_rate(Some(u64::MAX)), "18E");
    }

    #[test]
    fn find_interface_path_accepts_exact_name() {
        let root = tempfile_dir("exact");
        fs::create_dir(root.join("eth0")).unwrap();

        assert_eq!(
            find_interface_path(&root, "eth0").unwrap(),
            root.join("eth0")
        );
    }

    #[test]
    fn find_interface_path_accepts_ip_link_name() {
        let root = tempfile_dir("peer");
        fs::create_dir(root.join("eth0")).unwrap();

        assert_eq!(
            find_interface_path(&root, "eth0@if3").unwrap(),
            root.join("eth0")
        );
    }

    #[test]
    fn find_interface_path_rejects_bad_names() {
        let root = tempfile_dir("bad");

        assert!(find_interface_path(&root, "").is_err());
        assert!(find_interface_path(&root, "../eth0").is_err());
    }

    #[test]
    fn find_interface_path_rejects_missing_interfaces() {
        let root = tempfile_dir("missing");

        assert!(find_interface_path(&root, "eth0").is_err());
    }

    #[test]
    fn read_counters_reads_rx_and_tx_files() {
        let root = tempfile_dir("counters");
        let statistics = root.join("eth0").join("statistics");
        fs::create_dir_all(&statistics).unwrap();
        write_file(&statistics.join("rx_bytes"), "123\n");
        write_file(&statistics.join("tx_bytes"), "456\n");

        assert_eq!(
            read_counters(&root.join("eth0")).unwrap(),
            NetworkCounters {
                rx_bytes: 123,
                tx_bytes: 456
            }
        );
    }

    fn tempfile_dir(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("oled-network-test-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        root
    }

    fn write_file(path: &Path, value: &str) {
        let mut file = fs::File::create(path).unwrap();
        file.write_all(value.as_bytes()).unwrap();
    }
}
