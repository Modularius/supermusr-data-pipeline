mod linear_background;
mod back2back;
mod lorentz;
mod model;

use ceres_solver::{nlls_problem::{NllsProblem, NllsProblemSolution}, CostFunctionType, ParameterBlock, SolverOptions};

use crate::{alc_detector::fitting::{linear_background::LinearBackground, model::{Accumulator, Model}}, pulse_detection::Real};

fn calc_jacobian<'a, const N : usize, M>(time: &[Real], models: &[M], linear_background : LinearBackground, jacobians: &'a mut [Option<&'a mut [&'a mut [Real]]>])
    where M : Model<N, Context = ()>
{
            let (lin_back_jacobian, peak_jacobians) = jacobians
                .split_last_mut()
                .expect("Cost function `jacobians` argument should be non-empty, this should never fail.");
            
            let models_jacobians = Iterator::zip(models.into_iter(), peak_jacobians);
            for (model,jacobian) in models_jacobians {
                if let Some(jacobian) = jacobian {
                    model.accumulate_jacobian(time, jacobian);
                }
            }

            if let Some(jacobian) = lin_back_jacobian {
                linear_background.accumulate_jacobian(time, jacobian);
            }
}

fn cost_function<'a, const N : usize, M>(time: &'a [Real], intensities: &'a [Real], num_peaks: usize) -> CostFunctionType<'a>
    where M : Model<N, Context = ()>
{
    Box::new(
    move |parameters, residuals, jacobians| {
        assert_eq!(parameters.len(), num_peaks + 1);
        
        let models = (0..num_peaks)
            .map(|peak|M::new(parameters[peak]))
            .collect::<Vec<_>>();
        let linear_background = LinearBackground::new(parameters[num_peaks]);

        assert_eq!(time.len(), intensities.len());
        assert_eq!(residuals.len(), intensities.len());

        let residuals_intensities = Iterator::zip(residuals.iter_mut(), intensities.iter());
        for (res,intensity) in residuals_intensities {
            *res = -intensity;
        }

        for m in &models {
            m.accumulate_value(time, residuals);
        }
        linear_background.accumulate_value(time, residuals);

        if let Some(jacobians) = jacobians {
            assert_eq!(jacobians.len(), num_peaks + 1);
            calc_jacobian(time, &models, linear_background, jacobians)
        }
        true
    })
}

fn fit_n_peaks<'a, const N : usize, M>(time: &[Real], intensities: &[Real], num_peaks: usize) -> NllsProblemSolution 
    where M : Model<N, Context = ()>
{
    let (problem, _) = (0..num_peaks).fold(
        NllsProblem::new()
            .residual_block_builder(),
        |builder, _|
            builder.add_parameter(ParameterBlock::new(M::init_parameters(&())))
    )
    .add_parameter(ParameterBlock::new(LinearBackground::init_parameters(&())))
    .set_cost(cost_function::<N,M>(time, intensities, num_peaks), time.len())
    .build_into_problem()
    .expect("Problem should build, this should never fail.");

    let options = SolverOptions::builder()
        .build()
        .expect("Solver options should build, this should never fail.");

    let sol = problem.solve(&options)
        .expect("Problem should solve, this should never fail.");

    sol
}