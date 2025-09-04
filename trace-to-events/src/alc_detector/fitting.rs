use ceres_solver::nlls_problem::NllsProblem;

pub(crate) fn fitting(time: &[Real], intensities: &[Real]) {

}

struct B2bParams {
    i: Real,
    a: Real,
    b: Real,
    x0: Real,
    s: Real,
}

struct LinearBackgroundParams {
    m: Real,
    c: Real,
}

fn fit_n_peaks(time: &[Real], intensities: &[Real], num_peaks: usize) {
    let (mut problem,_) = NllsProblem::new()
        .residual_block_builder()
        .set_parameters(parameters)
        .build_into_problem()
        .unwrap();
}