mod linear_background;
mod back2back;
mod lorentz;

use ceres_solver::{nlls_problem::NllsProblem, types::JacobianType};

use crate::{alc_detector::fitting::linear_background::LinearBackground, pulse_detection::Real};

pub(crate) fn fitting(time: &[Real], intensities: &[Real]) {

}

pub(crate) trait Model<const N: usize> {
    fn accumulate_value(&self, input: &[Real], output: &mut [Real]);
    fn accumulate_jacobian(&self, input: &[Real], output: &mut [[Real; N]]);
}

struct CompoundModel<const N1: usize, const N2: usize, M1, M2>
    where M1: Model<N1>, M2: Model<N2>
{
    m1: M1,
    m2: M2,
}

const fn add<const N1: usize, const N2: usize>() -> usize {
    N1 + N2
}

impl<const N1: usize, const N2: usize, M1, M2> Model<{N1 + N2}> for CompoundModel<N1,N2, M1, M2>
    where M1: Model<N1>, M2: Model<N2> {

    fn accumulate_value(&self, input: &[Real], output: &mut [Real]) {
        self.m1.accumulate_value(input, output);
        self.m2.accumulate_value(input, output);
    }

    fn accumulate_jacobian(&self, input: &[Real], output: &mut [Self::Jacobian]) {
        todo!()
    }
}

struct MultiPeakModel<const N : usize, M : Model<N>> {
    models: Vec<M>,
    linear_background: LinearBackground
}

impl Model

fn calc_residuals<M : Model>(peak_models: &[M], linear_model: LinearBackground, time: &[Real], intensities: &[Real], residuals: &mut [Real]) {
    for (i,res) in residuals.iter_mut().enumerate() {
        *res = -intensities[i];
    }
    for m in peak_models {
        m.accumulate_value(time, residuals);
    }
    linear_model.accumulate_value(time, residuals);
}

fn calc_jacobian<M : Model>(peak_models: &[M], linear_model: LinearBackground, time: &[Real], intensities: &[Real], jacobian: &mut [(M::Jacobian, <LinearBackground as Model>::Jacobian)]) {
    for m in peak_models {
        m.accumulate_jacobian(time, jacobian.0);
    }
    linear_model.accumulate_jacobian(time, jacobian.1);
}

fn fit_n_peaks(time: &[Real], intensities: &[Real], num_peaks: usize) {
    let (mut problem,_) = NllsProblem::new()
        .residual_block_builder()
        //.set_parameters(parameters)
        .build_into_problem()
        .unwrap();
}