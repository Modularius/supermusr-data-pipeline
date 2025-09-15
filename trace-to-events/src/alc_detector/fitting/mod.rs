mod linear_background;
mod back2back;
mod lorentz;

use ceres_solver::nlls_problem::NllsProblem;

use crate::pulse_detection::Real;

pub(crate) fn fitting(time: &[Real], intensities: &[Real]) {

}

pub(crate) trait Model {
    type Params;
    
    fn accumulate_value();
    fn accumulate_jacobian();
}

fn fit_n_peaks(time: &[Real], intensities: &[Real], num_peaks: usize) {
    let (mut problem,_) = NllsProblem::new()
        .residual_block_builder()
        .set_parameters(parameters)
        .build_into_problem()
        .unwrap();
}