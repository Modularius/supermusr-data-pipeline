use super::{Real, Window};

#[derive(Default, Clone)]
pub(crate) struct AdaptiveBin {
    baseline: Real,
    value: Real,
    smoothing_factor: Real,
    warm_up: usize,
    time: usize,
}

impl AdaptiveBin {
    pub(crate) fn new(warm_up: usize, smoothing_factor: Real) -> Self {
        AdaptiveBin {
            warm_up,
            smoothing_factor,
            ..Default::default()
        }
    }
}

impl Window for AdaptiveBin {
    type TimeType = Real;
    type InputType = Real;
    type OutputType = Real;

    fn push(&mut self, value: Real) -> bool {
        self.value = value - self.baseline;
        if self.time < self.warm_up {
            self.baseline = if self.time == 0 {
                value
            } else {
                value * self.smoothing_factor + self.baseline * (1. - self.smoothing_factor)
            };
            self.time += 1;
            false
        } else {
            true
        }
    }

    fn output(&self) -> Option<Real> {
        (self.time == self.warm_up).then_some(self.value)
    }

    fn apply_time_shift(&self, time: Real) -> Real {
        time - (self.warm_up as Real)
    }
}



fn adaptive_bin(x : Vec<Time>, y : Vec<RealArray<2>>) {
    let dx = 1; //x[1] - x[0]
    x = x - 0.5*dx
    
    //adaptive bin
    
    new_x = [x[0]]
    new_y = []
    
    counts = y[0]*(x[1]-x[0])
    current_h = y[0]
    
    tol = 0.001;
    
    for j in range(1,len(x)-1) {
        counts += y[j]*(x[j+1] - x[j])   ;
        new_bin_width = x[j+1] - new_x[-1];
        new_h = counts/new_bin_width;
        expected_counts = current_h*new_bin_width;
    
        if np.abs(counts-expected_counts) > tol {
            new_x.append(x[j+1]);
            new_y.append(new_h);
            counts = 0;
        }
    
        current_h = new_h
    }
    # record the end point
    
    new_x.append(x[-1])
    
    new_y.append(y[-1])
}
