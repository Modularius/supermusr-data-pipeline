mod linear_background;
mod back2back;
mod lorentz;
mod model;

use std::marker::PhantomData;

use ceres_solver::{nlls_problem::NllsProblem, CostFunctionType, ParameterBlock};

use crate::{alc_detector::fitting::{linear_background::LinearBackground, model::{Accumulator, Model}}, pulse_detection::Real};

struct MultiPeakModel<'a, const N : usize, M> {
    models: Vec<M>,
    linear_background: LinearBackground<'a>,
    phantom: PhantomData<&'a()>
}

impl<'a, const N : usize, M>
    MultiPeakModel<'a, N,M>
    where
        M : Model<Jacobian = &'a mut [&'a mut [Real]]>
{
    fn accumulate_value(&self, input: &[Real], output: &mut [Real]) {
        for m in &self.models {
            m.accumulate_value(input, output);
        }
        self.linear_background.accumulate_value(input, output);
    }

    fn accumulate_jacobian(&self, input: &[Real], output: &'a mut [Option<&mut [&mut [f64]]>]) {
        for (i,m) in self.models.iter().enumerate() {
            if let Some(jacobian) = output.get(i).expect("") {
                m.accumulate_jacobian(input, jacobian);
            }
        }
        if let Some(jacobian) = output[self.models.len()] {
            self.linear_background.accumulate_jacobian(input, jacobian);
        }
    }
}

impl<'a, const N : usize, M> MultiPeakModel<'a, N,M> where M : Model<Context = (), Jacobian = &'a mut [Real], Parameters = [Real; N]> {
    fn new(num_peaks: usize, parameters: &[&[f64]]) -> Self {
        let models = (0..num_peaks).map(|peak|M::new(parameters[peak])).collect::<Vec<_>>();
        let linear_background = LinearBackground::new(parameters[num_peaks]);
        MultiPeakModel {
            models,
            linear_background,
            phantom: PhantomData
        }
    }
}
struct Fitting<'a> {
    num_peaks: usize,
    time: &'a [Real],
    intensities: &'a [Real],
}

impl<'a> Fitting<'a> {
    fn new(time: &'a [Real], intensities: &'a [Real], num_peaks: usize) -> Self {
        Fitting { time, intensities, num_peaks }
    }
    fn compute_residuals<M: Accumulator>(&self, model: &M, residuals: &mut [f64]) {
        for (i,res) in residuals.iter_mut().enumerate() {
            *res = -self.intensities[i];
        }
        model.accumulate_value(self.time, residuals);
    }

    fn compute_jacobian<'b, M>(&self, model: &M, jacobian: &mut [Option<&'b mut [&'b mut [f64]]>]) where M: Accumulator<Jacobian = Option<&'b mut [&'b mut [f64]]>> {
        model.accumulate_jacobian(self.time, jacobian);
    }
}

fn fit_n_peaks<const N : usize, M>(time: &[Real], intensities: &[Real], num_peaks: usize) where M : Model<Context = (), Parameters = [Real; N]> {
    let fitting = Fitting::new(time, intensities, num_peaks);
    let cost: CostFunctionType = Box::new(
    move |parameters, residuals, jacobian| {
        let model = MultiPeakModel::<'_, N,M>::new(num_peaks, parameters);

        fitting.compute_residuals(&model, residuals);
        if let Some(jacobian) = jacobian {
            fitting.compute_jacobian(&model, jacobian);
        }
        true
    });
    let problem = (0..num_peaks).fold(
        NllsProblem::new()
            .residual_block_builder(),
        |builder, _peak_index| {
            builder.add_parameter(ParameterBlock::new(M::init_parameters(&())))
        }
    )
    .add_parameter(ParameterBlock::new(LinearBackground::init_parameters(&())))
    .set_cost(cost, time.len())
    .build_into_problem()
    .unwrap();
}