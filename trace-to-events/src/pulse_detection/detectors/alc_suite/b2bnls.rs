/*
 * Back to Back Exponential nonlinear least squares interface to Ceres
 * See http://ceres-solver.org/
 *
 * Copyright (C) 2025 The Science and Technology Facilities Council (STFC)
 * Author: Jaroslav Fowkes (STFC)
 */
use ceres_solver;

use crate::pulse_detection::detectors::alc_suite::b2bexp::{sum_of_back_to_back_jacobian, sum_of_back_to_back_residuals, B2BParams, LinParams};


// B2B cost function with analytical derivatives
struct B2BFittingCost {
    x: Vec<f64>,
    y: Vec<f64>,
    npeaks: usize,
}

impl B2BFittingCost {
    fn B2BFittingCost(x: Vec<f64>, y: Vec<f64>, npeaks: usize) -> Self  {
        Self { x, y, npeaks }
    }

    fn Evaluate(&self, parameters : &[B2BParams],
                      residuals: &mut Vec<f64>,
                      jacobians: Option<&mut [Vec<f64>]>) -> bool {
        // Compute residuals
        sum_of_back_to_back_residuals(self.x, self.y, self.npeaks, parameters, residuals);

        // Compute jacobians
        if let Some(jacobians) = jacobians {
            sum_of_back_to_back_jacobian(self.x, self.npeaks, parameters, jacobians);
        }

        return true;
    }
}

impl ceres_solver::DynamicCostFunction for B2BFittingCost {
}


// B2B alternating cost function with analytical derivatives
struct B2BAlternatingFittingCost {
        x: Vec<f64>,
        y: Vec<f64>,
        npeaks: usize,
        nfree: usize,
        free_inds: Vec<usize>,  // number of free peak parameteres
        b2b_params: Vec<B2BParams>,  // indices of free peak parameters
        linear_params: LinParams,
        all_jacobians: Vec<Vec<f64>>  // !!FIXME: this is inefficient!!
};


impl B2BAlternatingFittingCost {
    // B2B alternating cost function with analytical derivatives: constructor to set required data
    fn new(x: Vec<f64>, y: Vec<f64>, npeaks: usize, nfree: usize, free_inds: Vec<usize>, b2b_params: Vec<B2BParams>, lin_params: LinParams) -> Self {

        // !!FIXME: this is inefficient!!
        // Allocate storage for jacobians
        let all_jacobians = Vec::<Vec<f64>>::new();
        all_jacobians = new double*[npeaks+1];
        for(int k = 0; k < npeaks; k++) all_jacobians[k] = new double[nx*5];
        all_jacobians[npeaks] = new double[nx*2];
        Self {
            x, y, npeaks, nfree, free_inds, 
        }
    }


    // B2B alternating cost function with analytical derivatives: compute residuals and jacobians
    bool B2BAlternatingFittingCost::Evaluate(double const* const* parameters,
                                            double* residuals,
                                            double** jacobians) const {

        // Update non-fixed parameter values
        for(int k = 0; k < npeaks; k++) {
            for(int i = 0; i < nfree; i++) {
                all_params[k][free_inds[i]] = parameters[k][i];
            }
        }
        for(int i = 0; i < 2; i++) {
            all_params[npeaks][i] = parameters[npeaks][i];
        }

        // Compute residuals
        sum_of_back_to_back_residuals(nx, x, y, npeaks, all_params, residuals);

        // Compute jacobians
        if(jacobians != nullptr) {
            sum_of_back_to_back_jacobian(nx, x, npeaks, all_params, all_jacobians);

            // !!FIXME: this is inefficient!!
            // Copy out jacobians for free parameters
            for(int k = 0; k < npeaks; k++) {
                for(int j = 0; j < nfree; j++) {
                    for(int i = 0; i < nx; i++) {
                        jacobians[k][i*nfree + j] = all_jacobians[k][i*5 + free_inds[j]];
                    }
                }
            }
            for(int j = 0; j < 2; j++) {
                for(int i = 0; i < nx; i++) {
                    jacobians[npeaks][i*2 + j] = all_jacobians[npeaks][i*2 + j];
                }
            }
        }

        return true;
    }

}


// // B2B cost functor with numerical derivatives (for testing, can be removed)
// class B2BCostFunctor {
//     public:
//         explicit B2BCostFunctor(int nx, double *x, double *y, int npeaks);
//         bool operator()(double const* const* parameters, double* residuals) const;

//     private:
//         int nx;
//         double *x;
//         double *y;
//         int npeaks;
// };