pub enum Base {
    Two,
    Ten,
}

const BINARY_PREFIXES: [&str; 9] = [
    "",
    "Ki",
    "Mi",
    "Gi",
    "Ti",
    "Pi",
    "Ei",
    "Zi",
    "Yi",
];

const METRIC_PREFIXES: [&str; 9] = [
    "",
    "K",
    "M",
    "G",
    "T",
    "P",
    "E",
    "Z",
    "Y",
];

pub struct GlancableSizesWithOrdersOfMagnitude {
    unit_short: String,
    value: u64,
}

impl GlancableSizesWithOrdersOfMagnitude {
    // todo: use float and allow decimal points?
    pub fn new(value: u64, base: Base) -> GlancableSizesWithOrdersOfMagnitude {
        if value == 0 {
            return Self {unit_short: "B".to_string(), value: 0};
        }
        
        let mut remainder = value;
        let mut previous_remainder = 0u64;
        let mut magnitude = 0u64;
        let (numerical_base, prefixes) = match base {
            Base::Two => (1024u64, BINARY_PREFIXES),
            Base::Ten => (1000u64, METRIC_PREFIXES),
        };

        while remainder >= numerical_base {
            previous_remainder = remainder;
            remainder /= numerical_base;
            magnitude += 1;
        }

        // if we have 0.9 or more, round up
        if previous_remainder - (remainder * numerical_base) > 900 {
            remainder += 1;
        }

        return Self {
            unit_short: format!("{}B", prefixes[magnitude as usize]),
            value: remainder,
        }
    }
}

impl std::fmt::Display for GlancableSizesWithOrdersOfMagnitude {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.value, self.unit_short)
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_kib() {
        let v = GlancableSizesWithOrdersOfMagnitude::new(4096, Base::Two);
        assert_eq!(v.value, 4);
        assert_eq!(v.unit_short, "KiB");
    }

    #[test]
    fn test_kb() {
        let v = GlancableSizesWithOrdersOfMagnitude::new(4000, Base::Ten);
        assert_eq!(v.value, 4);
        assert_eq!(v.unit_short, "KB");
    }

    #[test]
    fn test_6tb_metric() {
        let v = GlancableSizesWithOrdersOfMagnitude::new(6_001_000_443_904, Base::Ten);
        assert_eq!(v.value, 6);
        assert_eq!(v.unit_short, "TB");
    }

    #[test]
    fn test_6tb_binary() {
        let v = GlancableSizesWithOrdersOfMagnitude::new(6_001_000_443_904, Base::Two);
        assert_eq!(v.value, 5);
        assert_eq!(v.unit_short, "TiB");
    }

    #[test]
    fn test_little_less_than_4tb_metric() {
        let v = GlancableSizesWithOrdersOfMagnitude::new(3_920_320_420_904, Base::Ten);
        assert_eq!(v.value, 4);
        assert_eq!(v.unit_short, "TB");
    }
}
