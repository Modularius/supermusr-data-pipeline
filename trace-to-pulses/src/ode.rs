
use std::error::Error;

use na::{Matrix, ArrayStorage, MatrixXx2, MatrixXx1, RawStorage};
use nalgebra::{self as na, OMatrix, OVector, U2, U3};
use lstsq;
use anyhow::{Result,anyhow};

use crate::{Real, RealArray};

#[derive(Default)]
pub struct MonicQuadratic {
    quadratic : Real,
    linear : Real,
    constant : Real,
}

impl MonicQuadratic {
    pub fn new(quadratic : Real, linear : Real, constant : Real) -> Self { Self { quadratic, linear, constant } }
    pub fn get_coefficients(&self) -> (Real,Real,Real) { (self.quadratic, self.linear, self.constant) }
    pub fn discriminant(&self) -> Real {
        self.linear*self.linear - 4.*self.constant*self.quadratic
    }
    pub fn calc_solutions(&self) -> (Real,Real) {
        let discr_sqrt = self.discriminant().sqrt();
        ((-self.linear + discr_sqrt)*0.5/self.quadratic, (-self.linear - discr_sqrt)*0.5/self.quadratic)
    }
    pub fn calc_complex_solutions(&self) -> ((Real,Real),(Real,Real)) {
        let discr = self.discriminant();
        if discr < 0. {
            let discr_sqrt = (-discr).sqrt();
            ((-self.linear*0.5/self.quadratic, discr_sqrt*0.5/self.quadratic), (-self.linear*0.5/self.quadratic, -discr_sqrt*0.5/self.quadratic))
        } else {
            let discr_sqrt = discr.sqrt();
            (((-self.linear + discr_sqrt)*0.5/self.quadratic,0.), ((-self.linear - discr_sqrt)*0.5/self.quadratic,0.))
        }
    }
}

pub enum Status {
    Ok((MonicQuadratic,Real)),
    TooShort,
    DiscriminantNonPositive(Real),
    ParameterNonPositive(Real),
}

#[derive(Default, Clone)]
pub struct ParameterEstimator {
    y: Vec<Real>,
    dy: Vec<Real>,
    dy2: Vec<Real>,
}
/*impl Default for ParameterEstimator {
    fn default() -> Self {
        Self { a: Default::default(), b: Default::default() }
    }
}*/
//use ndarray::{array, Array1, Array2, ArrayView};
//use ndarray_linalg::{LeastSquaresSvd, LeastSquaresSvdInto, LeastSquaresSvdInPlace};

impl ParameterEstimator {
    pub fn push(&mut self, y: Real, dy: Real, dy2 : Real) {
        self.y.push(y);
        self.dy.push(dy);
        self.dy2.push(dy2);
    }
    pub fn clear(&mut self) {
        self.y.clear();
        self.dy.clear();
        self.dy2.clear();
    }
    pub fn get_parameters(&self) -> Result<Status> {
        if self.y.len() < 5 {
            return Ok(Status::TooShort);
        }
        let a_y = OVector::<f64, na::Dyn>::from_row_slice(self.y.as_slice());
        let a_dy = OVector::<f64, na::Dyn>::from_row_slice(self.dy.as_slice());
        let a_dy2 = OVector::<f64, na::Dyn>::from_row_slice(self.dy2.as_slice());
        let a = OMatrix::<Real, na::Dyn, U3>::from_columns(&[a_y, a_dy, a_dy2]);

        let b = OVector::<f64, na::Dyn>::from_row_slice(&vec![1.0; self.y.len()]);
        
        let epsilon = 1e-16;
        // -y'' = x.0 y + x.1 y'
        let sol = lstsq::lstsq(&a, &b, epsilon).map_err(|s|anyhow!(s))?;
        let x = MonicQuadratic::new(sol.solution[2], sol.solution[1], sol.solution[0]);
        Ok(Status::Ok((x,sol.residuals)))
    }
}



#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::processing;
    use super::*;


    fn biexp_peak_time(kappa : Real, rho : Real) -> Real {
        (Real::ln(kappa) - Real::ln(rho))/(1./rho - 1./kappa)
    }
    fn biexp_value(t : Real, kappa : Real, rho : Real) -> Real {
        (Real::exp(-t/kappa) - Real::exp(-t/rho)) + (rand::random::<Real>() - 0.5)*0.002
    }
    #[test]
    fn test_with_data() {
        let mut pe = ParameterEstimator::default();
        let mut peak = (0.0,0.0);
        let rho = 2.5;
        let kappa = 13.0;
        let dt = 1.0;
        let n = 10;
        let amplitude = 10.0;
        let y = (0..n).map(|i|amplitude*biexp_value((i as Real)*dt,kappa,rho)).collect_vec();
        for i in 1..(n - 1) {
            let dy = (y[i + 1] - y[i])/dt;
            let dy2 = (y[i - 1] + y[i + 1] - 2.0*y[i])/dt/dt;
            pe.push(y[i],dy,dy2);
            if y[i] > peak.1 {
                peak = (i as Real * dt, y[i]);
            }
        }
        if let Status::Ok(((xrho, xkappa),(xa,xb),res)) = pe.get_parameters().unwrap() {
            println!("{:?}",(xrho, xkappa));
            println!("{:?}",(xa, xb));
            println!("{:?}",(((y[0] + y[2] - 2.0*y[1])/dt)/(y[1] - y[0]), 1./xrho + 1./xkappa, 1./rho + 1./kappa));
            println!("{:?}",res);
            println!("{peak:?}");
            println!("{0},{1}",biexp_peak_time(kappa,rho),biexp_value(biexp_peak_time(kappa,rho),kappa,rho));
            println!("{0},{1}",biexp_peak_time(xkappa,xrho),biexp_value(biexp_peak_time(xkappa,xrho),xkappa,xrho));       
        }
    }
}