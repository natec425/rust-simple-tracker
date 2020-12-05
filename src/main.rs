use chrono::{Duration, Utc};
use clap::{App, Arg};
use yahoo_finance_api as yahoo;

fn min(series: &[f64]) -> Option<f64> {
    series
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .map(|n| *n)
}

fn max(series: &[f64]) -> Option<f64> {
    series
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .map(|n| *n)
}

fn n_window_sma(n: usize, series: &[f64]) -> Option<Vec<f64>> {
    if n > series.len() {
        None
    } else {
        Some(
            series
                .windows(n)
                .map(|subseries| subseries.iter().sum::<f64>() / n as f64)
                .collect(),
        )
    }
}

fn price_diff(series: &[f64]) -> Option<(f64, f64)> {
    match (series.first(), series.last()) {
        (Some(f), Some(l)) => {
            let abs = l - f;
            let rel = abs / f * 100.0;
            Some((rel, abs))
        }
        _ => None,
    }
}

fn main() {
    let matches = App::new("yahoo-api")
        .arg(Arg::with_name("symbol").takes_value(true).required(true))
        .arg(Arg::with_name("days").takes_value(true).required(true))
        .get_matches();

    let symbol = matches.value_of("symbol").unwrap();
    let from = matches
        .value_of("days")
        .and_then(|from| from.parse::<u32>().ok())
        .filter(|&from| from >= 30)
        .unwrap();
    let provider = yahoo::YahooConnector::new();

    match provider
        .get_quote_range(symbol, "1d", &format!("{}d", from))
        .and_then(|response| response.quotes())
    {
        Ok(quotes) => {
            let period_start = Utc::now() - Duration::days(from as i64);
            let close_prices = quotes.iter().map(|q| q.close).collect::<Vec<_>>();
            let price = close_prices.last().unwrap();
            let (rel, _) = price_diff(&close_prices).unwrap();
            let min = min(&close_prices).unwrap();
            let max = max(&close_prices).unwrap();
            let averages = n_window_sma(30, &close_prices).unwrap();
            let thirty_avg = averages.last().unwrap();
            println!(
                "{},{},${:.2},{:.2}%,${:.2},${:.2},${:.2}",
                period_start.to_rfc3339(),
                symbol,
                price,
                rel,
                min,
                max,
                thirty_avg
            );
        }
        Err(e) => println!("{:?}", e),
    };
}
