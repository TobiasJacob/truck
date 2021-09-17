#![allow(dead_code)]

use crate::loops_store::*;
use std::ops::Deref;
use truck_meshalgo::prelude::*;
use truck_topology::*;

trait PolylineBoundary {
	fn area(&self) -> f64;
	fn include(&self, c: Point2) -> bool;
}

impl PolylineBoundary for PolylineCurve<Point2> {
	fn area(&self) -> f64 {
		self.windows(2).fold(0.0, |res, p| {
			res + (p[1][0] + p[0][0]) * (p[1][1] - p[0][1])
		}) / 2.0
	}
	fn include(&self, c: Point2) -> bool {
		let t = 2.0 * std::f64::consts::PI * rand::random::<f64>();
		let r = Vector2::new(f64::cos(t), f64::sin(t));
		self.windows(2)
			.try_fold(0_i32, |counter, p| {
				let a = p[0] - c;
				let b = p[1] - c;
				let s0 = r[0] * a[1] - r[1] * a[0];
				let s1 = r[0] * b[1] - r[1] * b[0];
				let s2 = a[0] * b[1] - a[1] * b[0];
				let x = s2 / (s1 - s0);
				if x.so_small() && s0 * s1 < 0.0 {
					None
				} else if x > 0.0 && s0 <= 0.0 && s1 > 0.0 {
					Some(counter + 1)
				} else if x > 0.0 && s0 >= 0.0 && s1 < 0.0 {
					Some(counter - 1)
				} else {
					Some(counter)
				}
			})
			.map(|counter| counter > 0)
			.unwrap_or(false)
	}
}

fn create_parameter_boundary<P, C, S>(
	face: &Face<P, C, S>,
	wire: &Wire<P, C>,
	tol: f64,
) -> Option<PolylineCurve<Point2>>
where
	P: Copy,
	C: ParametricCurve<Point = P> + ParameterDivision1D,
	S: Clone + SearchParameter<Point = P, Parameter = (f64, f64)>,
{
	let surface = face.get_surface();
	let pt = wire.front_vertex().unwrap().get_point();
	let p: Point2 = surface.search_parameter(pt, None, 100)?.into();
	let vec = wire.edge_iter().try_fold(vec![p], |mut vec, edge| {
		let curve = edge.get_curve();
		let div = curve.parameter_division(curve.parameter_range(), tol);
		let mut p = *vec.last().unwrap();
		let closure = |t: f64| -> Option<Point2> {
			p = surface
				.search_parameter(curve.subs(t), Some(p.into()), 100)?
				.into();
			Some(p)
		};
		let add: Option<Vec<Point2>> = match edge.orientation() {
			true => div.into_iter().skip(1).map(closure).collect(),
			false => div.into_iter().rev().skip(1).map(closure).collect(),
		};
		vec.append(&mut add?);
		Some(vec)
	})?;
	Some(PolylineCurve(vec))
}

#[derive(Clone, Debug)]
struct WireChunk<'a, C> {
	poly: PolylineCurve<Point2>,
	wire: &'a BoundaryWire<Point3, C>,
}

fn divide_one_face<C, S>(
	face: &Face<Point3, C, S>,
	loops: &Loops<Point3, C>,
	tol: f64,
) -> Option<Vec<(Face<Point3, C, S>, BoundaryStatus)>>
where
	C: ParametricCurve<Point = Point3> + ParameterDivision1D,
	S: Clone + SearchParameter<Point = Point3, Parameter = (f64, f64)>,
{
	let (mut pre_faces, mut negative_wires) = (Vec::new(), Vec::new());
	loops.iter().try_for_each(|wire| {
		let poly = create_parameter_boundary(face, wire, tol)?;
		match poly.area() > 0.0 {
			true => pre_faces.push(vec![WireChunk { poly, wire }]),
			false => negative_wires.push(WireChunk { poly, wire }),
		}
		Some(())
	})?;
	negative_wires.into_iter().try_for_each(|chunk| {
		let pt = chunk.poly.front();
		let op = pre_faces.iter_mut().find(|face| face[0].poly.include(pt))?;
		op.push(chunk);
		Some(())
	})?;
	let vec: Vec<_> = pre_faces
		.into_iter()
		.map(|pre_face| {
			let surface = face.get_surface();
			let op = pre_face
				.iter()
				.find(|chunk| chunk.wire.status() != BoundaryStatus::Unknown);
			let status = match op {
				Some(chunk) => chunk.wire.status(),
				None => BoundaryStatus::Unknown,
			};
			let wires: Vec<Wire<Point3, C>> = pre_face
				.into_iter()
				.map(|chunk| chunk.wire.deref().clone())
				.collect();
			let mut new_face = Face::debug_new(wires, surface);
			if !face.orientation() {
				new_face.invert();
			}
			(new_face, status)
		})
		.collect();
	Some(vec)
}

#[derive(Clone, Debug, Default)]
pub struct DivideFacesResult<Shell> {
	and: Shell,
	or: Shell,
	unknown: Shell,
}

pub fn divide_faces<C, S>(
	shell: &Shell<Point3, C, S>,
	loops_store: &LoopsStore<Point3, C>,
	tol: f64,
) -> Option<DivideFacesResult<Shell<Point3, C, S>>>
where
	C: ParametricCurve<Point = Point3> + ParameterDivision1D,
	S: Clone + SearchParameter<Point = Point3, Parameter = (f64, f64)>,
{
	let mut res = DivideFacesResult::<Shell<Point3, C, S>>::default();
	shell
		.iter()
		.zip(loops_store)
		.try_for_each(|(face, loops)| {
			if loops
				.iter()
				.all(|wire| wire.status() == BoundaryStatus::Unknown)
			{
				res.unknown.push(face.clone());
			} else {
				let vec = divide_one_face(face, loops, tol)?;
				vec.into_iter().for_each(|(face, status)| match status {
					BoundaryStatus::And => res.and.push(face),
					BoundaryStatus::Or => res.or.push(face),
					BoundaryStatus::Unknown => res.unknown.push(face),
				});
			}
			Some(())
		})?;
	Some(res)
}

#[cfg(test)]
mod tests;
