use serde::Serialize;

#[derive(Debug, Clone, Default)]
pub struct Histogram<T> {
    pub data: Vec<T>,
    pub bins: Option<usize>,
}

impl<T> Histogram<T> {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn show(self)
    where
        T: Serialize + Clone + 'static,
    {
        self.plotly().show();
    }
    pub fn show_terminal(self) {
        todo!()
    }

    pub fn plotly(self) -> plotly::Plot
    where
        T: Serialize + Clone + 'static,
    {
        let mut histogram = plotly::Histogram::default().x(self.data);
        if let Some(bins) = self.bins {
            histogram = histogram.n_bins_x(bins);
        }
        let mut plot = plotly::Plot::new();
        plot.add_trace(histogram);
        plot
    }
}
