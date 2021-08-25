use serde::{Deserialize, Serialize};
use truck_base::cgmath64::*;
use truck_meshalgo::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntersectionCurve<C, S> {
	surface0: S,
	surface1: S,
	leader: C,
	tol: f64,
}

pub fn double_projection<S>(
	surface0: &S,
	hint0: Option<(f64, f64)>,
	surface1: &S,
	hint1: Option<(f64, f64)>,
	point: Point3,
	normal: Vector3,
	trials: usize,
) -> Option<(Point3, Point2, Point2)>
where
	S: ParametricSurface3D + SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>,
{
	if trials == 0 {
		return None;
	}
	let (u0, v0) = surface0.search_nearest_parameter(point, hint0, 10)?;
	let pt0 = surface0.subs(u0, v0);
	let (u1, v1) = surface1.search_nearest_parameter(point, hint1, 10)?;
	let pt1 = surface1.subs(u1, v1);
	if point.near(&pt0) && point.near(&pt1) && pt0.near(&pt1) {
		Some((point, Point2::new(u0, v0), Point2::new(u1, v1)))
	} else {
		let n0 = surface0.normal(u0, v0);
		let n1 = surface1.normal(u1, v1);
		let mat = Matrix3::from_cols(n0, n1, normal).transpose();
		let inv = mat.invert()?;
		let pt = inv * Vector3::new(pt0.dot(n0), pt1.dot(n1), point.dot(normal));
		double_projection(
			surface0,
			Some((u0, v0)),
			surface1,
			Some((u1, v1)),
			Point3::from_vec(pt),
			normal,
			trials - 1,
		)
	}
}

impl<C, S> IntersectionCurve<C, S> {
	/// This curve is a part of intersection of `self.surface0()` and `self.surface1()`.
	#[inline(always)]
	pub fn surface0(&self) -> &S { &self.surface0 }
	/// This curve is a part of intersection of `self.surface0()` and `self.surface1()`.
	#[inline(always)]
	pub fn surface1(&self) -> &S { &self.surface1 }
	/// Returns the polyline leading this curve.
	#[inline(always)]
	pub fn leader(&self) -> &C { &self.leader }
	/// Returns the polyline leading this curve.
	#[inline(always)]
	pub(super) fn leader_mut(&mut self) -> &mut C { &mut self.leader }
	/// The tolerance for generating this intersection curve.
	#[inline(always)]
	pub fn tolerance(&self) -> f64 { self.tol }
}

impl<S> IntersectionCurve<PolylineCurve<Point3>, S>
where
	S: ParametricSurface3D + SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>
{
	pub(super) fn try_new(
		surface0: S,
		surface1: S,
		poly: PolylineCurve<Point3>,
		tol: f64,
	) -> Option<Self> {
		let mut polyline = PolylineCurve(Vec::new());
		for p in poly.windows(2) {
			let n = (p[1] - p[0]).normalize();
			let q = double_projection(&surface0, None, &surface1, None, p[0], n, 100)?.0;
			polyline.push(q);
		}
		let q = if poly[0].near(&poly[poly.len() - 1]) {
			polyline[0]
		} else {
			let n = (poly[poly.len() - 1] - poly[poly.len() - 2]).normalize();
			double_projection(
				&surface0,
				None,
				&surface1,
				None,
				poly[poly.len() - 1],
				n,
				100,
			)?.0
		};
		polyline.push(q);
		Some(Self {
			surface0,
			surface1,
			leader: polyline,
			tol,
		})
	}

	pub fn remeshing(&mut self) -> bool {
		let div = algo::curve::parameter_division(self, self.parameter_range(), self.tol);
		let mut polyline = PolylineCurve(Vec::new());
		for t in div {
			let pt = self.leader().subs(t);
			let normal = self.leader().der(t);
			let (pt, _, _) = match double_projection(
				self.surface0(),
				None,
				self.surface1(),
				None,
				pt,
				normal,
				100,
			) {
				Some(got) => got,
				None => return false,
			};
			polyline.push(pt);
		}
		self.leader = polyline;
		true
	}
}

impl<C, S> ParametricCurve for IntersectionCurve<C, S>
where
	C: ParametricCurve<Point = Point3, Vector = Vector3>,
	S: ParametricSurface3D + SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>
{
	type Point = Point3;
	type Vector = Vector3;
	#[inline(always)]
	fn parameter_range(&self) -> (f64, f64) { self.leader.parameter_range() }
	fn subs(&self, t: f64) -> Point3 {
		double_projection(
			&self.surface0,
			None,
			&self.surface1,
			None,
			self.leader.subs(t),
			self.leader.der(t),
			100,
		)
		.unwrap()
		.0
	}
	fn der(&self, t: f64) -> Vector3 {
		let n = self.leader.der(t);
		let (_, p0, p1) = double_projection(
			&self.surface0,
			None,
			&self.surface1,
			None,
			self.leader.subs(t),
			n.normalize(),
			100,
		)
		.unwrap();
		let d = self
			.surface0
			.normal(p0.x, p0.y)
			.cross(self.surface1.normal(p1.x, p1.y))
			.normalize();
		d * (n.dot(n) / d.dot(n))
	}
	/// This method is unimplemented! Should panic!!
	#[inline(always)]
	fn der2(&self, _: f64) -> Vector3 {
		unimplemented!();
	}
}

impl<C, S> ParameterDivision1D for IntersectionCurve<C, S>
where
	C: ParametricCurve<Point = Point3, Vector = Vector3>,
	S: ParametricSurface3D + SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>
{
	#[inline(always)]
	fn parameter_division(&self, range: (f64, f64), tol: f64) -> Vec<f64> {
		algo::curve::parameter_division(self, range, tol)
	}
}

impl<C, S> Cut for IntersectionCurve<C, S>
where
	C: Cut<Point = Point3, Vector = Vector3>,
	S: ParametricSurface3D + SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>
{
	#[inline(always)]
	fn cut(&mut self, t: f64) -> Self {
		Self {
			surface0: self.surface0.clone(),
			surface1: self.surface1.clone(),
			leader: self.leader.cut(t),
			tol: self.tol,
		}
	}
}

impl<C: Invertible, S: Clone> Invertible for IntersectionCurve<C, S> {
	fn invert(&mut self) {
		self.leader.invert();
	}
}

impl<C, S> SearchParameter for IntersectionCurve<C, S>
where
	C: ParametricCurve<Point = Point3, Vector = Vector3> + SearchNearestParameter<Point = Point3, Parameter = f64>,
	S: ParametricSurface3D + SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>
{
	type Point = Point3;
	type Parameter = f64;
	fn search_parameter(&self, point: Point3, hint: Option<f64>, trials: usize) -> Option<f64> {
		let t = self
			.leader()
			.search_nearest_parameter(point, hint, trials)
			.unwrap();
		let pt = self.subs(t);
		match pt.near(&point) {
			true => Some(t),
			false => None,
		}
	}
}

/// Only derive from leading curve. Not precise.
impl<C, S> SearchNearestParameter for IntersectionCurve<C, S>
where
	C: ParametricCurve<Point = Point3, Vector = Vector3> + SearchNearestParameter<Point = Point3, Parameter = f64>,
	S: ParametricSurface3D + SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>
{
	type Point = Point3;
	type Parameter = f64;
	fn search_nearest_parameter(&self, point: Point3, hint: Option<f64>, trials: usize) -> Option<f64> {
		self.leader().search_nearest_parameter(point, hint, trials)
	}
}
pub fn intersection_curves<S>(
	surface0: S,
	polygon0: &PolygonMesh,
	surface1: S,
	polygon1: &PolygonMesh,
	tol: f64,
) -> Vec<(PolylineCurve<Point3>, Option<IntersectionCurve<PolylineCurve<Point3>, S>>)>
where
	S: ParametricSurface3D + SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>,
{
	let interferences = polygon0.extract_interference(polygon1);
	let polylines = crate::polyline_construction::construct_polylines(&interferences);
	polylines
		.into_iter()
		.map(|polyline| {
			let curve = IntersectionCurve::try_new(
				surface0.clone(),
				surface1.clone(),
				polyline.clone(),
				tol,
			);
			(polyline, curve)
		})
		.collect()
}

#[cfg(test)]
mod tests;