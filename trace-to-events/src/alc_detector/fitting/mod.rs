mod linear_background;
mod back2back;
mod lorentz;
mod model;

pub(super) use back2back::Back2BackExpParams;
pub(super) use model::Model;

use ceres_solver::{nlls_problem::{NllsProblem, NllsProblemSolution}, solver::{DenseLinearAlgebraLibraryType, LinearSolverType, MinimizerType, TrustRegionStrategyType}, CostFunctionType, ParameterBlock, SolverOptions};

use crate::{alc_detector::fitting::{linear_background::{LinearBackground, LinearBackgroundParams}, model::{Accumulator, Jacobian}}, pulse_detection::Real};

fn calc_jacobian<'a, M: Accumulator>(time: &[Real], models: &[M], linear_background : LinearBackground, jacobians: &'a mut [Option<Jacobian<'a>>]) {
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

fn init_residuals<'a>(intensities: &'a [Real], residuals: &'a mut [Real]) {
    assert_eq!(residuals.len(), intensities.len());

    let residuals_intensities = Iterator::zip(residuals.iter_mut(), intensities.iter());
    for (res,intensity) in residuals_intensities {
        *res = -intensity;
    }
}

fn cost_function<'a, M: Model>(time: &'a [Real], intensities: &'a [Real], num_peaks: usize) -> CostFunctionType<'a> {
    Box::new( move |parameters, residuals, jacobians| {
        init_residuals(intensities, residuals);
        
        assert_eq!(parameters.len(), num_peaks + 1);
        let models = (0..num_peaks)
            .map(|peak|M::new(parameters[peak]).accumulator())
            .collect::<Vec<_>>();
        let linear_background = LinearBackgroundParams::accumulator(LinearBackgroundParams::new(parameters[num_peaks]));

        assert_eq!(time.len(), intensities.len());
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

#[tracing::instrument(skip_all)]
pub(super) fn fit_n_peaks<'a, M>(time: &[Real], intensities: &[Real], num_peaks: usize) -> NllsProblemSolution 
    where M: Model<Context = (Real,Real)>
{
    let bounds = (time.first().cloned().unwrap_or_default(), time.last().cloned().unwrap_or_default());

    let (problem, _) = (0..num_peaks).fold(
        NllsProblem::new()
            .residual_block_builder(),
        |builder, _| {
            let mut parameter_block = ParameterBlock::new(M::init_parameters(&bounds));
            parameter_block.set_lower_bounds(M::lower_bounds(&bounds)).set_upper_bounds(M::upper_bounds(&bounds));
            builder.add_parameter(parameter_block)
        }
    )
    .add_parameter(ParameterBlock::new(LinearBackgroundParams::init_parameters(&())))
    .set_cost(cost_function::<M>(time, intensities, num_peaks), time.len())
    .build_into_problem()
    .expect("Problem should build, this should never fail.");

    let options = SolverOptions::builder()
        .max_num_iterations(100)
        .minimizer_type(MinimizerType::TRUST_REGION)
        .trust_region_strategy_type(TrustRegionStrategyType::LEVENBERG_MARQUARDT)
        .dense_linear_algebra_library_type(DenseLinearAlgebraLibraryType::LAPACK)
        .linear_solver_type(LinearSolverType::DENSE_NORMAL_CHOLESKY)
        .minimizer_progress_to_stdout(true)
        .build()
        .expect("Solver options should build, this should never fail.");

    let sol = problem.solve(&options)
        .expect("Problem should solve, this should never fail.");
    sol
}
