use trade_common::indicators::ma;

pub const DEFAULT_FAST_MA_PERIOD: usize = 7;
pub const DEFAULT_SLOW_MA_PERIOD: usize = 25;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SignalParams {
    pub fast_ma_period: usize,
    pub slow_ma_period: usize,
}

impl Default for SignalParams {
    fn default() -> Self {
        Self {
            fast_ma_period: DEFAULT_FAST_MA_PERIOD,
            slow_ma_period: DEFAULT_SLOW_MA_PERIOD,
        }
    }
}

impl SignalParams {
    pub fn minimum_bars(self) -> usize {
        self.slow_ma_period + 1
    }
}

#[derive(Clone, Copy)]
pub struct SignalContext {
    pub side: &'static str,
    pub fast_ma: f64,
    pub slow_ma: f64,
}

pub fn detect_signal(closes: &[f64]) -> Option<SignalContext> {
    detect_signal_with_params(closes, SignalParams::default())
}

pub fn detect_signal_with_params(
    closes: &[f64],
    params: SignalParams,
) -> Option<SignalContext> {
    if closes.len() < params.minimum_bars() {
        return None;
    }

    let ma7 = ma(closes, params.fast_ma_period);
    let ma25 = ma(closes, params.slow_ma_period);

    let (prev_ma7, curr_ma7) = (ma7[ma7.len() - 2], ma7[ma7.len() - 1]);
    let (prev_ma25, curr_ma25) = (ma25[ma25.len() - 2], ma25[ma25.len() - 1]);

    let (Some(p7), Some(c7), Some(p25), Some(c25)) = (prev_ma7, curr_ma7, prev_ma25, curr_ma25)
    else {
        return None;
    };

    if p7 <= p25 && c7 > c25 {
        return Some(SignalContext {
            side: "BUY",
            fast_ma: c7,
            slow_ma: c25,
        });
    }

    if p7 >= p25 && c7 < c25 {
        return Some(SignalContext {
            side: "SELL",
            fast_ma: c7,
            slow_ma: c25,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{detect_signal, detect_signal_with_params, SignalParams};

    #[test]
    fn detect_signal_returns_none_when_data_is_insufficient() {
        let closes = vec![100.0; 25];
        assert!(detect_signal(&closes).is_none());
    }

    #[test]
    fn detect_signal_detects_buy_cross_on_last_bar() {
        let mut closes = vec![100.0; 24];
        closes.push(90.0);
        closes.push(200.0);

        let signal = detect_signal(&closes).unwrap();

        assert_eq!(signal.side, "BUY");
        assert!(signal.fast_ma > signal.slow_ma);
    }

    #[test]
    fn detect_signal_detects_sell_cross_on_last_bar() {
        let mut closes = vec![100.0; 24];
        closes.push(110.0);
        closes.push(0.0);

        let signal = detect_signal(&closes).unwrap();

        assert_eq!(signal.side, "SELL");
        assert!(signal.fast_ma < signal.slow_ma);
    }

    #[test]
    fn detect_signal_supports_custom_ma_periods() {
        let closes = vec![100.0, 100.0, 100.0, 99.0, 98.0, 110.0];
        let params = SignalParams {
            fast_ma_period: 2,
            slow_ma_period: 4,
        };

        let signal = detect_signal_with_params(&closes, params).unwrap();

        assert_eq!(signal.side, "BUY");
        assert!(signal.fast_ma > signal.slow_ma);
    }
}