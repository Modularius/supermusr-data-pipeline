use common::Intensity;
use rand::*;
use itertools::Itertools;
//use std::num::sqrt;

#[derive(Default)]
struct DSPEngine {

}

pub fn mean(data : &Vec<Intensity>) -> f32 {
    data.iter().sum::<Intensity>() as f32/data.len() as f32
}
pub fn std(data : &Vec<Intensity>) -> f32 {
    let mean = mean(data);
    let sum_of_squares = data.iter().map(|x|(*x as f32).powi(2)).sum::<f32>();
    let var = (sum_of_squares - mean.powi(2)*data.len() as f32)/(data.len() as f32 - 1.);
    var.sqrt()
}
pub fn range(data : &Vec<Intensity>) -> f32 {
    *data.iter().max().unwrap() as f32 - *data.iter().min().unwrap() as f32
}

pub fn smooth(mut data : Vec<Intensity>, radius : usize) -> Vec<Intensity> {
    let mut value = data.iter().take(2*radius).sum::<Intensity>() as f32/(2.*radius as f32);
    for i in 0..radius {
        data[i] = value as Intensity;
    }
    for i in radius..(data.len()-radius) {
        value += (data[i + radius] as f32 - data[i - radius] as f32)/(2.*radius as f32);
        data[i] = value as Intensity;
    }
    for i in (data.len()-radius)..data.len() {
        data[i] = value as Intensity;
    }
    data
}

pub fn scale(data : Vec<Intensity>, factor : f32) -> Vec<Intensity> {
    let mean = mean(&data);
    data.into_iter().map(|x| (((x as f32 - mean)*factor) + mean) as Intensity).collect()
}

pub fn rescale(data : Vec<Intensity>, new_std : f32) -> Vec<Intensity> {
    let old_std = std(&data);
    scale(data, new_std/old_std)
}

pub fn find_peaks(data : &Vec<Intensity>) -> Vec<usize> {
    data.iter().enumerate().tuple_windows().map(|(l,m,r)| if l.1 < m.1 && m.1 > r.1 { Some(m.0) } else { None }).flatten().collect()
}

pub fn select(data : &Vec<Intensity>, indices : Vec<usize>) -> Vec<(usize,Intensity)> {
    indices.into_iter().map(|i| (i,data[i])).collect()
}

pub fn vector_to_point_data(data : &Vec<Intensity>) -> Vec<(f32,f32)> {
    data.iter().enumerate().map(|(i,val)|(i as f32, *val as f32)).collect()
}