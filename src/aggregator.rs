use simple::AveragePrice;

/// Averge price aggregator ds
pub struct Aggregator {
    averages: Vec<AveragePrice>,
}

impl Aggregator {
    pub fn new() -> Self {
        Aggregator {
            averages: Vec::new(),
        }
    }

    /// Adds an average to the aggregator.
    pub fn add_average(&mut self, average: AveragePrice) {
        self.averages.push(average);
    }

    /// Computes the Final Average of all the clients
    pub fn final_average(&mut self) -> f64 {
        if self.averages.is_empty() {
            0.0
        } else {
            self.averages
                .iter()
                .map(|avg| avg.average_price)
                .sum::<f64>()
        }
    }
}
