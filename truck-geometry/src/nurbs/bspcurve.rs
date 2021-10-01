use super::*;
use crate::errors::Error;
use std::ops::*;

impl<P> BSplineCurve<P> {
    /// constructor.
    /// # Arguments
    /// * `knot_vec` - the knot vector
    /// * `control_points` - the vector of the control points
    /// # Panics
    /// Panics occurs if:
    /// * There are no control points.
    /// * The number of knots is more than the one of control points.
    /// * The range of the knot vector is zero.
    pub fn new(knot_vec: KnotVec, control_points: Vec<P>) -> BSplineCurve<P> {
        BSplineCurve::try_new(knot_vec, control_points).unwrap_or_else(|e| panic!("{}", e))
    }

    /// constructor.
    /// # Arguments
    /// * `knot_vec` - the knot vector
    /// * `control_points` - the vector of the control points
    /// # Failures
    /// * If there are no control points, returns [`Error::EmptyControlPoint<f64>s`].
    /// * If the number of knots is more than the one of control points, returns [`Error::TooShortKnotVector`].
    /// * If the range of the knot vector is zero, returns [`Error::ZeroRange`].
    ///
    /// [`Error::EmptyControlPoint<f64>s`]: errors/enum.Error.html#variant.EmptyControlPoint<f64>s
    /// [`Error::TooShortKnotVector`]: errors/enum.Error.html#variant.TooShortKnotVector
    /// [`Error::ZeroRange`]: errors/enum.Error.html#variant.ZeroRange
    pub fn try_new(knot_vec: KnotVec, control_points: Vec<P>) -> Result<BSplineCurve<P>> {
        if control_points.is_empty() {
            Err(Error::EmptyControlPoints)
        } else if knot_vec.len() <= control_points.len() {
            Err(Error::TooShortKnotVector(
                knot_vec.len(),
                control_points.len(),
            ))
        } else if knot_vec.range_length().so_small() {
            Err(Error::ZeroRange)
        } else {
            Ok(BSplineCurve::new_unchecked(knot_vec, control_points))
        }
    }

    /// constructor.
    /// # Arguments
    /// * `knot_vec` - the knot vector
    /// * `control_points` - the vector of the control points
    /// # Remarks
    /// This method is prepared only for performance-critical development and is not recommended.  
    /// This method does NOT check the rules for constructing B-spline curve.  
    /// The programmer must guarantee these conditions before using this method.
    #[inline(always)]
    pub const fn new_unchecked(knot_vec: KnotVec, control_points: Vec<P>) -> BSplineCurve<P> {
        Self {
            knot_vec,
            control_points,
        }
    }

    /// constructor.
    /// # Arguments
    /// * `knot_vec` - the knot vector
    /// * `control_points` - the vector of the control points
    /// # Remarks
    /// This method checks the rules for constructing B-spline curve in the debug mode.  
    /// The programmer must guarantee these conditions before using this method.
    #[inline(always)]
    pub fn debug_new(knot_vec: KnotVec, control_points: Vec<P>) -> BSplineCurve<P> {
        match cfg!(debug_assertions) {
            true => Self::new(knot_vec, control_points),
            false => Self::new_unchecked(knot_vec, control_points),
        }
    }

    /// Returns the reference of the knot vector
    #[inline(always)]
    pub fn knot_vec(&self) -> &KnotVec { &self.knot_vec }

    /// Returns the `idx`th knot
    #[inline(always)]
    pub fn knot(&self, idx: usize) -> f64 { self.knot_vec[idx] }

    /// Returns the reference of the control points.
    #[inline(always)]
    pub fn control_points(&self) -> &Vec<P> { &self.control_points }

    /// Returns the reference of the control point corresponding to the index `idx`.
    #[inline(always)]
    pub fn control_point(&self, idx: usize) -> &P { &self.control_points[idx] }

    /// Returns the mutable reference of the control point corresponding to index `idx`.
    #[inline(always)]
    pub fn control_point_mut(&mut self, idx: usize) -> &mut P { &mut self.control_points[idx] }
    /// Returns the iterator on all control points
    #[inline(always)]
    pub fn control_points_mut(&mut self) -> impl Iterator<Item = &mut P> {
        self.control_points.iter_mut()
    }

    /// Apply the given transformation to all control points.
    #[inline(always)]
    pub fn transform_control_points<F: FnMut(&mut P)>(&mut self, f: F) {
        self.control_points.iter_mut().for_each(f)
    }

    /// Returns the degree of B-spline curve
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector2::new(1.0, 2.0), Vector2::new(2.0, 3.0), Vector2::new(3.0, 4.0)];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// assert_eq!(bspcurve.degree(), 2);
    /// ```
    #[inline(always)]
    pub fn degree(&self) -> usize { self.knot_vec.len() - self.control_points.len() - 1 }
    /// Inverts a curve
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::uniform_knot(2, 2);
    /// let ctrl_pts = vec![Vector2::new(1.0, 2.0), Vector2::new(2.0, 3.0), Vector2::new(3.0, 4.0), Vector2::new(4.0, 5.0)];
    /// let bspcurve0 = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let mut bspcurve1 = bspcurve0.clone();
    /// bspcurve1.invert();
    ///
    /// const N: usize = 100; // sample size
    /// for i in 0..=N {
    ///     let t = (i as f64) / (N as f64);
    ///     assert_near2!(bspcurve0.subs(t), bspcurve1.subs(1.0 - t));
    /// }
    /// ```
    #[inline(always)]
    pub fn invert(&mut self) -> &mut Self {
        self.knot_vec.invert();
        self.control_points.reverse();
        self
    }

    /// Returns whether the knot vector is clamped or not.
    #[inline(always)]
    pub fn is_clamped(&self) -> bool { self.knot_vec.is_clamped(self.degree()) }

    /// Normalizes the knot vector  
    #[inline(always)]
    pub fn knot_normalize(&mut self) -> &mut Self {
        self.knot_vec.try_normalize().unwrap();
        self
    }

    /// Translates the knot vector
    #[inline(always)]
    pub fn knot_translate(&mut self, x: f64) -> &mut Self {
        self.knot_vec.translate(x);
        self
    }
}

impl<P: ControlPoint<f64>> BSplineCurve<P> {
    /// Returns the closure of substitution.
    /// # Examples
    /// The following test code is the same test with the one of `BSplineCurve::subs()`.
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::from(vec![-1.0, -1.0, -1.0, 1.0, 1.0, 1.0]);
    /// let ctrl_pts = vec![Vector2::new(-1.0, 1.0), Vector2::new(0.0, -1.0), Vector2::new(1.0, 1.0)];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    ///
    /// const N: usize = 100; // sample size
    /// let get_t = |i: usize| -1.0 + 2.0 * (i as f64) / (N as f64);
    /// let res: Vec<_> = (0..=N).map(get_t).map(bspcurve.get_closure()).collect();
    /// let ans: Vec<_> = (0..=N).map(get_t).map(|t| Vector2::new(t, t * t)).collect();
    /// res.iter().zip(&ans).for_each(|(v0, v1)| assert_near2!(v0, v1));
    /// ```
    #[inline(always)]
    pub fn get_closure(&self) -> impl Fn(f64) -> P + '_ { move |t| self.subs(t) }
    #[inline(always)]
    fn delta_control_points(&self, i: usize) -> P::Diff {
        if i == 0 {
            self.control_point(i).to_vec()
        } else if i == self.control_points.len() {
            self.control_points[i - 1].to_vec() * (-1.0)
        } else {
            self.control_points[i] - self.control_points[i - 1]
        }
    }
    #[inline(always)]
    fn delta2_control_points(&self, i: usize) -> P::Diff {
        let k = self.degree();
        let knot_vec = self.knot_vec();
        if i == 0 {
            let coef = inv_or_zero(knot_vec[i + k] - knot_vec[i]);
            self.delta_control_points(i) * coef
        } else if i == self.control_points.len() + 1 {
            let coef = inv_or_zero(knot_vec[i - 1 + k] - knot_vec[i - 1]);
            self.delta_control_points(i - 1) * (-coef)
        } else {
            let coef0 = inv_or_zero(knot_vec[i + k] - knot_vec[i]);
            let coef1 = inv_or_zero(knot_vec[i - 1 + k] - knot_vec[i - 1]);
            self.delta_control_points(i) * coef0 - self.delta_control_points(i - 1) * coef1
        }
    }
    /// Returns the derived B-spline curve.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, 0.0), Vector2::new(1.0, 1.0)];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let derived = bspcurve.derivation();
    ///
    /// // `bpscurve = (t, t^2), derived = (1, 2t)`
    /// const N : usize = 100; // sample size
    /// for i in 0..=N {
    ///     let t = 1.0 / (N as f64) * (i as f64);
    ///     assert_near2!(derived.subs(t), Vector2::new(1.0, 2.0 * t));
    /// }
    /// ```
    pub fn derivation(&self) -> BSplineCurve<P::Diff> {
        let n = self.control_points.len();
        let k = self.degree();
        let knot_vec = self.knot_vec.clone();
        let mut new_points = Vec::with_capacity(n + 1);
        if k > 0 {
            let (knot_vec, new_points) = (&knot_vec, &mut new_points);
            (0..=n).for_each(move |i| {
                let delta = knot_vec[i + k] - knot_vec[i];
                let coef = (k as f64) * inv_or_zero(delta);
                new_points.push(self.delta_control_points(i) * coef);
            });
        } else {
            new_points = vec![P::Diff::zero(); n];
        }
        BSplineCurve::new_unchecked(knot_vec, new_points)
    }
    pub(super) fn sub_near_as_curve<F: Fn(&P, &P) -> bool>(
        &self,
        other: &BSplineCurve<P>,
        div_coef: usize,
        ord: F,
    ) -> bool {
        if !self.knot_vec.same_range(&other.knot_vec) {
            return false;
        }

        let division = std::cmp::max(self.degree(), other.degree()) * div_coef;
        for i in 0..(self.knot_vec.len() - 1) {
            let delta = self.knot_vec[i + 1] - self.knot_vec[i];
            if delta.so_small() {
                continue;
            }

            for j in 0..division {
                let t = self.knot_vec[i] + delta * (j as f64) / (division as f64);
                if !ord(&self.subs(t), &other.subs(t)) {
                    return false;
                }
            }
        }
        true
    }
}

impl<P: ControlPoint<f64>> ParametricCurve for BSplineCurve<P> {
    type Point = P;
    type Vector = P::Diff;
    /// Substitutes to B-spline curve.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::from(vec![-1.0, -1.0, -1.0, 1.0, 1.0, 1.0]);
    /// let ctrl_pts = vec![Vector2::new(-1.0, 1.0), Vector2::new(0.0, -1.0), Vector2::new(1.0, 1.0)];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    ///
    /// // bspcurve coincides with (t, t * t) in the range [-1.0..1.0].
    /// const N: usize = 100; // sample size
    /// for i in 0..=N {
    ///     let t = -1.0 + 2.0 * (i as f64) / (N as f64);
    ///     assert_near2!(bspcurve.subs(t), Vector2::new(t, t * t));
    /// }
    /// ```
    #[inline(always)]
    fn subs(&self, t: f64) -> P {
        let basis = self
            .knot_vec
            .try_bspline_basis_functions(self.degree(), t)
            .unwrap();
        self.control_points
            .iter()
            .zip(basis)
            .fold(P::origin(), |sum, (pt, basis)| sum + pt.to_vec() * basis)
    }
    /// Substitutes to the derived B-spline curve.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, 0.0), Vector2::new(1.0, 1.0)];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    ///
    /// // `bpscurve = (t, t^2), derived = (1, 2t)`
    /// const N : usize = 100; // sample size
    /// for i in 0..=N {
    ///     let t = 1.0 / (N as f64) * (i as f64);
    ///     assert_near2!(bspcurve.der(t), Vector2::new(1.0, 2.0 * t));
    /// }
    /// ```
    #[inline(always)]
    fn der(&self, t: f64) -> P::Diff {
        let k = self.degree();
        let knot_vec = self.knot_vec();
        let closure = move |sum: P::Diff, (i, b): (usize, f64)| {
            let coef = inv_or_zero(knot_vec[i + k] - knot_vec[i]);
            sum + self.delta_control_points(i) * b * coef
        };
        knot_vec
            .bspline_basis_functions(k - 1, t)
            .into_iter()
            .enumerate()
            .fold(P::Diff::zero(), closure)
            * k as f64
    }
    /// Substitutes to the 2nd-ord derived B-spline curve.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::bezier_knot(3);
    /// let ctrl_pts = vec![
    ///     Vector2::new(0.0, 0.0),
    ///     Vector2::new(1.0, 1.0),
    ///     Vector2::new(0.0, 1.0),
    ///     Vector2::new(1.0, 0.0),
    /// ];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    ///
    /// // bpscurve = (4t^3 - 6t^2 + 3t, -3t^2 + 3t), derived2 = (24t - 12, -6)
    /// const N : usize = 100; // sample size
    /// for i in 0..=N {
    ///     let t = 1.0 / (N as f64) * (i as f64);
    ///     assert_near2!(bspcurve.der2(t), Vector2::new(24.0 * t - 12.0, -6.0));
    /// }
    /// ```
    #[inline(always)]
    fn der2(&self, t: f64) -> P::Diff {
        let k = self.degree();
        if k < 2 {
            return P::Diff::zero();
        }
        let knot_vec = self.knot_vec();
        let closure = move |sum: P::Diff, (i, b): (usize, f64)| {
            let coef = inv_or_zero(knot_vec[i + k - 1] - knot_vec[i]);
            sum + self.delta2_control_points(i) * b * coef
        };
        knot_vec
            .bspline_basis_functions(k - 2, t)
            .into_iter()
            .enumerate()
            .fold(P::Diff::zero(), closure)
            * k as f64
            * (k - 1) as f64
    }
    #[inline(always)]
    fn parameter_range(&self) -> (f64, f64) {
        (self.knot_vec[0], self.knot_vec[self.knot_vec.len() - 1])
    }
}

impl<P: ControlPoint<f64> + Tolerance> BSplineCurve<P> {
    /// Returns whether all control points are the same or not.
    /// If the knot vector is clamped, it means whether the curve is constant or not.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    ///
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let pt = Vector2::new(1.0, 2.0);
    /// let mut ctrl_pts = vec![pt.clone(), pt.clone(), pt.clone()];
    /// let const_bspcurve = BSplineCurve::new(knot_vec.clone(), ctrl_pts.clone());
    /// assert!(const_bspcurve.is_const());
    ///
    /// ctrl_pts.push(Vector2::new(2.0, 3.0));
    /// let bspcurve = BSplineCurve::new(knot_vec.clone(), ctrl_pts.clone());
    /// assert!(!bspcurve.is_const());
    /// ```
    /// # Remarks
    /// If the knot vector is not clamped and the BSpline basis function is not partition of unity,
    /// then perhaps returns true even if the curve is not constant.
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::uniform_knot(1, 5);
    /// let ctrl_pts = vec![Vector2::new(1.0, 2.0), Vector2::new(1.0, 2.0)];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    ///
    /// // bspcurve is not constant.
    /// assert_eq!(bspcurve.subs(0.0), Vector2::new(0.0, 0.0));
    /// assert_ne!(bspcurve.subs(0.5), Vector2::new(0.0, 0.0));
    ///
    /// // bspcurve.is_const() is true
    /// assert!(bspcurve.is_const());
    /// ```
    pub fn is_const(&self) -> bool {
        self.control_points
            .iter()
            .all(move |vec| vec.near(&self.control_points[0]))
    }

    /// Adds a knot `x`, and do not change `self` as a curve.  
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector2::new(-1.0, 1.0), Vector2::new(0.0, -1.0), Vector2::new(1.0, 1.0)];
    /// let mut bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let org_curve = bspcurve.clone();
    ///
    /// // add 4 knots
    /// bspcurve.add_knot(0.5).add_knot(0.5).add_knot(0.25).add_knot(0.75);
    /// assert_eq!(bspcurve.knot_vec().len(), org_curve.knot_vec().len() + 4);
    /// // bspcurve does not change as a curve
    /// assert!(bspcurve.near2_as_curve(&org_curve));
    /// ```
    /// # Remarks
    /// If the added knot `x` is out of the range of the knot vector, then the knot vector will extended.
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector2::new(-1.0, 1.0), Vector2::new(0.0, -1.0), Vector2::new(1.0, 1.0)];
    /// let mut bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// assert_eq!(bspcurve.knot_vec().range_length(), 1.0);
    /// assert_eq!(bspcurve.front(), Vector2::new(-1.0, 1.0));
    /// assert_eq!(bspcurve.back(), Vector2::new(1.0, 1.0));
    ///
    /// // add knots out of the range of the knot vectors.
    /// bspcurve.add_knot(-1.0).add_knot(2.0);
    /// assert_eq!(bspcurve.knot_vec().range_length(), 3.0);
    /// assert_eq!(bspcurve.front(), Vector2::new(0.0, 0.0));
    /// assert_eq!(bspcurve.back(), Vector2::new(0.0, 0.0));
    /// ```
    pub fn add_knot(&mut self, x: f64) -> &mut Self {
        if x < self.knot_vec[0] {
            self.knot_vec.add_knot(x);
            self.control_points.insert(0, P::origin());
            return self;
        }

        let k = self.degree();
        let n = self.control_points.len();

        let idx = self.knot_vec.add_knot(x);
        let start = if idx > k { idx - k } else { 0 };
        let end = if idx > n {
            self.control_points.push(P::origin());
            n + 1
        } else {
            self.control_points
                .insert(idx - 1, self.control_point(idx - 1).clone());
            idx
        };
        for i in start..end {
            let i0 = end + start - i - 1;
            let delta = self.knot_vec[i0 + k + 1] - self.knot_vec[i0];
            let a = (self.knot_vec[idx] - self.knot_vec[i0]) * inv_or_zero(delta);
            let p = self.delta_control_points(i0) * (1.0 - a);
            self.control_points[i0] -= p;
        }
        self
    }

    /// Removes a knot corresponding to the indices `idx`, and do not change `self` as a curve.
    /// If cannot remove the knot, do not change `self` and return `self`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector2::new(-1.0, 1.0), Vector2::new(0.0, -1.0), Vector2::new(1.0, 1.0)];
    /// let mut bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let org_curve = bspcurve.clone();
    ///
    /// // add knots and remove them.
    /// bspcurve.add_knot(0.5).add_knot(0.5).add_knot(0.25).add_knot(0.75);
    /// bspcurve.remove_knot(3).remove_knot(3).remove_knot(3).remove_knot(3);
    /// assert!(bspcurve.near2_as_curve(&org_curve));
    /// assert_eq!(bspcurve.knot_vec().len(), org_curve.knot_vec().len())
    /// ```
    pub fn remove_knot(&mut self, idx: usize) -> &mut Self {
        let _ = self.try_remove_knot(idx);
        self
    }

    /// Removes a knot corresponding to the indice `idx`, and do not change `self` as a curve.  
    /// If the knot cannot be removed, returns
    /// [`Error::CannotRemoveKnot`](./errors/enum.Error.html#variant.CannotRemoveKnot).
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use errors::Error;
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector2::new(-1.0, 1.0), Vector2::new(0.0, -1.0), Vector2::new(1.0, 1.0)];
    /// let mut bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let org_curve = bspcurve.clone();
    /// bspcurve.add_knot(0.5).add_knot(0.5).add_knot(0.25).add_knot(0.75);
    /// assert!(bspcurve.try_remove_knot(3).is_ok());
    /// assert_eq!(bspcurve.try_remove_knot(2), Err(Error::CannotRemoveKnot(2)));
    /// ```
    pub fn try_remove_knot(&mut self, idx: usize) -> Result<&mut BSplineCurve<P>> {
        let k = self.degree();
        let n = self.control_points.len();
        let knot_vec = &self.knot_vec;

        if idx < k + 1 || idx >= n {
            return Err(Error::CannotRemoveKnot(idx));
        }

        let mut new_points = Vec::with_capacity(k + 1);
        new_points.push(self.control_point(idx - k - 1).clone());
        for i in (idx - k)..idx {
            let delta = knot_vec[i + k + 1] - knot_vec[i];
            let a = inv_or_zero(delta) * (knot_vec[idx] - knot_vec[i]);
            if a.so_small() {
                break;
            } else {
                let p = *new_points.last().unwrap();
                let p = p + (self.control_points[i] - p) / a;
                new_points.push(p);
            }
        }

        if !new_points.last().unwrap().near(self.control_point(idx)) {
            return Err(Error::CannotRemoveKnot(idx));
        }

        for (i, vec) in new_points.into_iter().skip(1).enumerate() {
            self.control_points[idx - k + i] = vec;
        }

        self.control_points.remove(idx);
        self.knot_vec.remove(idx);
        Ok(self)
    }

    /// elevate 1 degree for bezier curve.
    fn elevate_degree_bezier(&mut self) -> &mut Self {
        let k = self.degree();
        self.knot_vec.add_knot(self.knot_vec[0]);
        self.knot_vec
            .add_knot(self.knot_vec[self.knot_vec.len() - 1]);
        self.control_points.push(P::origin());
        for i in 0..=(k + 1) {
            let i0 = k + 1 - i;
            let a = (i0 as f64) / ((k + 1) as f64);
            let p = self.delta_control_points(i0) * a;
            self.control_points[i0] = self.control_points[i0] - p;
        }
        self
    }

    /// elevate 1 degree.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::bezier_knot(1);
    /// let ctrl_pts = vec![Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0)];
    /// let mut bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// bspcurve.elevate_degree();
    /// assert_eq!(bspcurve.degree(), 2);
    /// assert_eq!(bspcurve.knot_vec(), &KnotVec::bezier_knot(2));
    /// assert_eq!(bspcurve.control_point(1), &Vector2::new(0.5, 0.5));
    /// ```
    pub fn elevate_degree(&mut self) -> &mut Self {
        let mut result = CurveCollector::Singleton;
        for mut bezier in self.bezier_decomposition() {
            result.concat(bezier.elevate_degree_bezier());
        }
        *self = result.unwrap();
        self
    }

    /// Makes the B-spline curve clamped
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::from(vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);
    /// let ctrl_pts = vec![Vector2::new(0.0, 1.0), Vector2::new(1.0, 2.0), Vector2::new(2.0, 3.0)];
    /// let mut bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// assert!(!bspcurve.is_clamped());
    /// bspcurve.clamp();
    /// assert!(bspcurve.is_clamped());
    /// assert_eq!(bspcurve.knot_vec().len(), 10);
    /// ```
    #[inline(always)]
    pub fn clamp(&mut self) -> &mut Self {
        let degree = self.degree();

        let s = self.knot_vec.multiplicity(0);
        for _ in s..=degree {
            self.add_knot(self.knot_vec[0]);
        }

        let n = self.knot_vec.len();
        let s = self.knot_vec.multiplicity(n - 1);
        for _ in s..=degree {
            self.add_knot(self.knot_vec[n - 1]);
        }
        self
    }

    /// Repeats `Self::try_remove_knot()` from the back knot in turn until the knot cannot be removed.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    ///
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector2::new(1.0, 2.0), Vector2::new(2.0, 3.0), Vector2::new(3.0, 4.0)];
    /// let mut bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let org_curve = bspcurve.clone();
    ///
    /// // add 4 new knots
    /// bspcurve.add_knot(0.5).add_knot(0.5).add_knot(0.25).add_knot(0.75);
    /// assert_eq!(bspcurve.knot_vec().len(), KnotVec::bezier_knot(2).len() + 4);
    ///
    /// // By the optimization, added knots are removed.
    /// bspcurve.optimize();
    /// assert_eq!(bspcurve.knot_vec(), &KnotVec::bezier_knot(2));
    /// assert!(bspcurve.near2_as_curve(&org_curve));
    /// ```
    pub fn optimize(&mut self) -> &mut Self {
        loop {
            let n = self.knot_vec.len();
            let closure = |flag, i| flag && self.try_remove_knot(n - i).is_err();
            if (1..=n).fold(true, closure) {
                break;
            }
        }
        self
    }

    /// Makes two splines having the same degrees.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    ///
    /// let knot_vec0 = KnotVec::bezier_knot(1);
    /// let ctrl_pts0 = vec![Vector2::new(1.0, 2.0), Vector2::new(2.0, 3.0)];
    /// let mut bspcurve0 = BSplineCurve::new(knot_vec0, ctrl_pts0);
    /// let knot_vec1 = KnotVec::bezier_knot(2);
    /// let ctrl_pts1 = vec![Vector2::new(1.0, 2.0), Vector2::new(2.0, 3.0), Vector2::new(3.0, 4.0)];
    /// let mut bspcurve1 = BSplineCurve::new(knot_vec1, ctrl_pts1);
    /// assert_ne!(bspcurve0.degree(), bspcurve1.degree());
    ///
    /// let org_curve0 = bspcurve0.clone();
    /// let org_curve1 = bspcurve1.clone();
    /// bspcurve0.syncro_degree(&mut bspcurve1);
    /// assert_eq!(bspcurve0.degree(), bspcurve1.degree());
    /// assert!(bspcurve0.near2_as_curve(&org_curve0));
    /// assert!(bspcurve1.near2_as_curve(&org_curve1));
    /// ```
    pub fn syncro_degree(&mut self, other: &mut Self) {
        let (degree0, degree1) = (self.degree(), other.degree());
        for _ in degree0..degree1 {
            self.elevate_degree();
        }
        for _ in degree1..degree0 {
            other.elevate_degree();
        }
    }

    /// Makes two splines having the same normalized knot vectors.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    ///
    /// let knot_vec0 = KnotVec::from(vec![0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0]);
    /// let ctrl_pts0 = vec![Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0), Vector2::new(2.0, 2.0), Vector2::new(3.0, 3.0)];
    /// let mut bspcurve0 = BSplineCurve::new(knot_vec0, ctrl_pts0);
    /// let mut org_curve0 = bspcurve0.clone();
    /// let knot_vec1 = KnotVec::from(vec![0.0, 0.0, 1.0, 3.0, 4.0, 4.0]);
    /// let ctrl_pts1 = vec![Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0), Vector2::new(2.0, 2.0), Vector2::new(3.0, 3.0)];
    /// let mut bspcurve1 = BSplineCurve::new(knot_vec1, ctrl_pts1);
    /// let mut org_curve1 = bspcurve1.clone();
    ///
    /// bspcurve0.syncro_knots(&mut bspcurve1);
    ///
    /// // The knot vectors are made the same.
    /// assert_eq!(bspcurve0.knot_vec(), bspcurve1.knot_vec());
    /// assert_eq!(
    ///     bspcurve0.knot_vec().as_slice(),
    ///     &[0.0, 0.0, 0.0, 0.25, 0.5, 0.75, 1.0, 1.0, 1.0]
    /// );
    /// // The degrees are not changed.
    /// assert_eq!(bspcurve0.degree(), org_curve0.degree());
    /// assert_eq!(bspcurve1.degree(), org_curve1.degree());
    /// // The knot vector is normalized, however, the shape of curve is not changed.
    /// assert!(bspcurve0.near2_as_curve(org_curve0.knot_normalize()));
    /// assert!(bspcurve1.near2_as_curve(org_curve1.knot_normalize()));
    /// ```
    pub fn syncro_knots(&mut self, other: &mut BSplineCurve<P>) {
        self.knot_normalize();
        other.knot_normalize();

        let mut i = 0;
        let mut j = 0;
        while !self.knot(i).near2(&1.0) || !other.knot(j).near2(&1.0) {
            if self.knot(i) - other.knot(j) > TOLERANCE {
                self.add_knot(other.knot(j));
            } else if other.knot(j) - self.knot(i) > TOLERANCE {
                other.add_knot(self.knot(i));
            }
            i += 1;
            j += 1;
        }

        if self.knot_vec.len() < other.knot_vec.len() {
            for _ in 0..(other.knot_vec.len() - self.knot_vec.len()) {
                self.add_knot(1.0);
            }
        } else if other.knot_vec.len() < self.knot_vec.len() {
            for _ in 0..(self.knot_vec.len() - other.knot_vec.len()) {
                other.add_knot(1.0);
            }
        }
    }

    /// Separates `self` into Bezier curves by each knots.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    ///
    /// let knot_vec = KnotVec::uniform_knot(2, 2);
    /// let ctrl_pts = vec![Vector2::new(0.0, 1.0), Vector2::new(1.0, 2.0), Vector2::new(2.0, 3.0), Vector2::new(3.0, 4.0)];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let beziers = bspcurve.bezier_decomposition();
    ///
    /// const N: usize = 100;
    /// for i in 0..=N {
    ///     let t = 0.5 * (i as f64) / (N as f64);
    ///     assert_near2!(bspcurve.subs(t), beziers[0].subs(t));
    /// }
    /// for i in 0..=N {
    ///     let t = 0.5 + 0.5 * (i as f64) / (N as f64);
    ///     assert_near2!(bspcurve.subs(t), beziers[1].subs(t));
    /// }
    /// ```
    pub fn bezier_decomposition(&self) -> Vec<BSplineCurve<P>> {
        let mut bspline = self.clone();
        bspline.clamp();
        let (knots, _) = self.knot_vec.to_single_multi();
        let n = knots.len();

        let mut result = Vec::new();
        for i in 2..n {
            result.push(bspline.cut(knots[n - i]));
        }
        result.push(bspline);
        result.reverse();
        result
    }

    /// Makes the curve locally injective.
    /// # Example
    /// ```
    /// use truck_geometry::*;
    /// const N : usize = 100; // sample size for test
    ///
    /// let knot_vec = KnotVec::from(
    ///     vec![0.0, 0.0, 0.0, 1.0, 3.0, 4.0, 4.0, 4.0]
    /// );
    /// let ctrl_pts = vec![
    ///     Vector3::new(1.0, 0.0, 0.0),
    ///     Vector3::new(0.0, 1.0, 0.0),
    ///     Vector3::new(0.0, 1.0, 0.0),
    ///     Vector3::new(0.0, 1.0, 0.0),
    ///     Vector3::new(0.0, 0.0, 1.0),
    /// ];
    ///
    /// let mut bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let mut flag = false;
    /// for i in 0..=N {
    ///     let t = 4.0 * (i as f64) / (N as f64);
    ///     flag = flag || bspcurve.subs(t).near(&bspcurve.subs(t + 1.0 / (N as f64)));
    /// }
    /// // There exists t such that bspcurve(t) == bspcurve(t + 0.01).
    /// assert!(flag);
    ///
    /// bspcurve.make_locally_injective().knot_normalize();
    /// let mut flag = false;
    /// for i in 0..=N {
    ///     let t = 1.0 * (i as f64) / (N as f64);
    ///     flag = flag || bspcurve.subs(t).near(&bspcurve.subs(t + 1.0 / (N as f64)));
    /// }
    /// // There does not exist t such that bspcurve(t) == bspcurve(t + 0.01).
    /// assert!(!flag);
    /// ```
    /// # Remarks
    /// If `self` is a constant curve, then does nothing.
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::from(vec![0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 2.0]);
    /// let ctrl_pts = vec![Vector2::new(1.0, 1.0); 4];
    /// let mut bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let org_curve = bspcurve.clone();
    /// bspcurve.make_locally_injective();
    /// assert_eq!(bspcurve, org_curve);
    /// ```
    pub fn make_locally_injective(&mut self) -> &mut Self {
        let mut iter = self.bezier_decomposition().into_iter();
        while let Some(bezier) = iter.next() {
            if !bezier.is_const() {
                *self = bezier;
                break;
            }
        }
        let mut x = 0.0;
        for mut bezier in iter {
            if bezier.is_const() {
                x += bezier.knot_vec.range_length();
            } else {
                self.concat(bezier.knot_translate(-x));
            }
        }
        self
    }
    /// Determine whether `self` and `other` is near as the B-spline curves or not.  
    ///
    /// Divides each knot interval into the number of degree equal parts,
    /// and check `|self(t) - other(t)| < TOLERANCE` for each end points `t`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::from(
    ///     vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 4.0, 4.0]
    /// );
    /// let ctrl_pts = vec![
    ///     Vector2::new(1.0, 1.0),
    ///     Vector2::new(3.0, 2.0),
    ///     Vector2::new(2.0, 3.0),
    ///     Vector2::new(4.0, 5.0),
    ///     Vector2::new(5.0, 4.0),
    ///     Vector2::new(1.0, 1.0),
    /// ];
    /// let bspcurve0 = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let mut bspcurve1 = bspcurve0.clone();
    /// assert!(bspcurve0.near_as_curve(&bspcurve1));
    /// *bspcurve1.control_point_mut(1) += Vector2::new(0.01, 0.0002);
    /// assert!(!bspcurve0.near_as_curve(&bspcurve1));
    /// ```
    #[inline(always)]
    pub fn near_as_curve(&self, other: &BSplineCurve<P>) -> bool {
        self.sub_near_as_curve(other, 1, |x, y| x.near(y))
    }

    /// Determines `self` and `other` is near in square order as the B-spline curves or not.  
    ///
    /// Divide each knot interval into the number of degree equal parts,
    /// and check `|self(t) - other(t)| < TOLERANCE`for each end points `t`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let eps = TOLERANCE;
    /// let knot_vec = KnotVec::from(
    ///     vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 4.0, 4.0]
    /// );
    /// let ctrl_pts = vec![
    ///     Vector2::new(1.0, 1.0),
    ///     Vector2::new(3.0, 2.0),
    ///     Vector2::new(2.0, 3.0),
    ///     Vector2::new(4.0, 5.0),
    ///     Vector2::new(5.0, 4.0),
    ///     Vector2::new(1.0, 1.0),
    /// ];
    /// let bspcurve0 = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let mut bspcurve1 = bspcurve0.clone();
    /// assert!(bspcurve0.near_as_curve(&bspcurve1));
    /// *bspcurve1.control_point_mut(1) += Vector2::new(eps, 0.0);
    /// assert!(!bspcurve0.near2_as_curve(&bspcurve1));
    /// ```
    #[inline(always)]
    pub fn near2_as_curve(&self, other: &BSplineCurve<P>) -> bool {
        self.sub_near_as_curve(other, 1, |x, y| x.near2(y))
    }
}

impl<P: ControlPoint<f64>> ParameterTransform for BSplineCurve<P> {
    #[inline(always)]
    fn parameter_transform(&mut self, scalar: f64, r#move: f64) -> &mut Self {
        self.knot_vec.transform(scalar, r#move);
        self
    }
}

#[test]
fn parameter_transform_random_test() {
    let curve = BSplineCurve::new(
        KnotVec::uniform_knot(4, 4),
        (0..8)
            .map(|_| {
                Point3::new(
                    rand::random::<f64>(),
                    rand::random::<f64>(),
                    rand::random::<f64>(),
                )
            })
            .collect(),
    );
    truck_geotrait::parameter_transform_random_test(&curve, 10);
}

impl<P: ControlPoint<f64> + Tolerance> Cut for BSplineCurve<P> {
    fn cut(&mut self, mut t: f64) -> BSplineCurve<P> {
        let degree = self.degree();

        let idx = match self.knot_vec.floor(t) {
            Some(idx) => idx,
            None => {
                let bspline = self.clone();
                let knot_vec = KnotVec::from(vec![t, self.knot_vec[0]]);
                let ctrl_pts = vec![P::origin()];
                *self = BSplineCurve::new(knot_vec, ctrl_pts);
                return bspline;
            }
        };
        let s = if t.near(&self.knot_vec[idx]) {
            t = self.knot_vec[idx];
            self.knot_vec.multiplicity(idx)
        } else {
            0
        };

        for _ in s..=degree {
            self.add_knot(t);
        }

        let k = self.knot_vec.floor(t).unwrap();
        let m = self.knot_vec.len();
        let n = self.control_points.len();
        let knot_vec0 = self.knot_vec.sub_vec(0..=k);
        let knot_vec1 = self.knot_vec.sub_vec((k - degree)..m);
        let control_points0 = Vec::from(&self.control_points[0..(k - degree)]);
        let control_points1 = Vec::from(&self.control_points[(k - degree)..n]);
        *self = BSplineCurve::new_unchecked(knot_vec0, control_points0);
        BSplineCurve::new_unchecked(knot_vec1, control_points1)
    }
}

#[test]
fn cut_random_test() {
    let curve = BSplineCurve::new(
        KnotVec::uniform_knot(4, 4),
        (0..8)
            .map(|_| {
                Point3::new(
                    rand::random::<f64>(),
                    rand::random::<f64>(),
                    rand::random::<f64>(),
                )
            })
            .collect(),
    );
    truck_geotrait::cut_random_test(&curve, 10);
}

impl<P: ControlPoint<f64> + Tolerance> Concat<BSplineCurve<P>> for BSplineCurve<P> {
    type Output = BSplineCurve<P>;
    /// Concats two B-spline curves.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// ```
    /// # Failure
    /// If the back of the knot vector of `self` does not coincides with the front of the one of `other`,
    /// ```
    /// use truck_geometry::*;
    /// use truck_geotrait::traits::ConcatError;
    ///
    /// let knot_vec0 = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
    /// let ctrl_pts0 = vec![Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0)];
    /// let mut bspcurve0 = BSplineCurve::new(knot_vec0, ctrl_pts0);
    /// let knot_vec1 = KnotVec::from(vec![2.0, 2.0, 3.0, 3.0]);
    /// let ctrl_pts1 = vec![Vector2::new(1.0, 1.0), Vector2::new(2.0, 2.0)];
    /// let mut bspcurve1 = BSplineCurve::new(knot_vec1, ctrl_pts1);
    ///
    /// assert_eq!(bspcurve0.try_concat(&mut bspcurve1), Err(ConcatError::DisconnectedParameters(1.0, 2.0)));
    /// ```
    fn try_concat(&self, other: &BSplineCurve<P>) -> std::result::Result<Self, ConcatError<P>> {
        let mut curve0 = self.clone();
        let mut curve1 = other.clone();
        curve0.syncro_degree(&mut curve1);
        curve0.clamp();
        curve1.clamp();
        curve0
            .knot_vec
            .try_concat(&curve1.knot_vec, curve0.degree())
            .map_err(|err| match err {
                Error::DifferentBackFront(a, b) => ConcatError::DisconnectedParameters(a, b),
                _ => unreachable!(),
            })?;
        let front = curve0.control_points.last().unwrap();
        let back = curve1.control_points.first().unwrap();
        if !front.near(back) {
            return Err(ConcatError::DisconnectedPoints(*front, *back));
        }
        curve0.control_points.extend(curve1.control_points);
        Ok(curve0)
    }
}

#[test]
fn concat_positive_test() {
    let mut part0 = BSplineCurve::new(
        KnotVec::uniform_knot(4, 4),
        (0..8)
            .map(|_| {
                Point3::new(
                    rand::random::<f64>(),
                    rand::random::<f64>(),
                    rand::random::<f64>(),
                )
            })
            .collect(),
    );
    let part1 = part0.cut(0.56);
    concat_random_test(&part0, &part1, 10);
}

#[test]
fn concat_negative_test() {
    let curve0 = BSplineCurve::new(
        KnotVec::bezier_knot(1),
        vec![Point2::new(0.0, 0.0), Point2::new(0.0, 1.0)],
    );
    let mut curve1 = BSplineCurve::new(
        KnotVec::bezier_knot(1),
        vec![Point2::new(1.0, 1.0), Point2::new(1.0, 1.0)],
    );
    assert_eq!(
        curve0.try_concat(&curve1),
        Err(ConcatError::DisconnectedParameters(1.0, 0.0))
    );
    curve1.knot_translate(1.0);
    assert_eq!(
        curve0.try_concat(&curve1),
        Err(ConcatError::DisconnectedPoints(
            Point2::new(0.0, 1.0),
            Point2::new(1.0, 1.0)
        ))
    );
}

impl<P> ParameterDivision1D for BSplineCurve<P>
where P: ControlPoint<f64>
        + EuclideanSpace<Scalar = f64, Diff = <P as ControlPoint<f64>>::Diff>
        + MetricSpace<Metric = f64>
{
    type Point = P;
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<P>) {
        algo::curve::parameter_division(self, range, tol)
    }
}

impl<P> BSplineCurve<P>
where
    P: ControlPoint<f64>
        + EuclideanSpace<Scalar = f64, Diff = <P as ControlPoint<f64>>::Diff>
        + MetricSpace<Metric = f64>
        + Tolerance,
    <P as ControlPoint<f64>>::Diff: InnerSpace<Scalar = f64> + Tolerance,
{
    /// Determines whether `self` is an arc of `curve` by repeating applying Newton method.
    ///
    /// The parameter `hint` is the init value, required that `curve.subs(hint)` is the front point of `self`.
    ///
    /// If `self` is an arc of `curve`, then returns `Some(t)` such that `curve.subs(t)` coincides with
    /// the back point of `self`. Otherwise, returns `None`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::from(
    ///     vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 3.0]
    /// );
    /// let ctrl_pts = vec![
    ///     Point3::new(0.0, 0.0, 0.0),
    ///     Point3::new(1.0, 0.0, 0.0),
    ///     Point3::new(1.0, 1.0, 0.0),
    ///     Point3::new(0.0, 1.0, 0.0),
    ///     Point3::new(0.0, 1.0, 1.0),
    /// ];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    ///
    /// let mut part = bspcurve.clone().cut(0.6);
    /// part.cut(2.8);
    /// let t = part.is_arc_of(&bspcurve, 0.6).unwrap();
    /// assert_near!(t, 2.8);
    ///
    /// // hint is required the init value.
    /// assert!(part.is_arc_of(&bspcurve, 0.7).is_none());
    ///
    /// // normal failure
    /// *part.control_point_mut(2) += Vector3::new(1.0, 2.0, 3.0);
    /// assert!(part.is_arc_of(&bspcurve, 0.6).is_none());
    /// ```
    pub fn is_arc_of(&self, curve: &BSplineCurve<P>, mut hint: f64) -> Option<f64> {
        let degree = std::cmp::max(self.degree(), curve.degree()) * 3 + 1;
        let (knots, _) = self.knot_vec.to_single_multi();
        if !self.subs(knots[0]).near(&curve.subs(hint)) {
            return None;
        }

        for i in 1..knots.len() {
            let range = knots[i] - knots[i - 1];
            for j in 1..=degree {
                let t = knots[i - 1] + range * (j as f64) / (degree as f64);
                let pt = ParametricCurve::subs(self, t);
                let res = curve.search_nearest_parameter(pt, Some(hint), 100);
                let flag = res.map(|res| hint <= res && curve.subs(res).near(&pt));
                hint = match flag {
                    Some(true) => res.unwrap(),
                    _ => return None,
                };
            }
        }
        Some(hint)
    }
}
impl<P> SearchNearestParameter for BSplineCurve<P>
where
    P: ControlPoint<f64>
        + EuclideanSpace<Scalar = f64, Diff = <P as ControlPoint<f64>>::Diff>
        + MetricSpace<Metric = f64>
        + Tolerance,
    <P as ControlPoint<f64>>::Diff: InnerSpace<Scalar = f64> + Tolerance,
{
    type Point = P;
    type Parameter = f64;
    /// Searches the parameter `t` which minimize |self(t) - point| by Newton's method with initial guess `hint`.
    /// Returns `None` if the number of attempts exceeds `trial` i.e. if `trial == 0`, then the trial is only one time.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::from(
    ///     vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 3.0]
    /// );
    /// let ctrl_pts = vec![
    ///     Point3::new(0.0, 0.0, 0.0),
    ///     Point3::new(1.0, 0.0, 0.0),
    ///     Point3::new(1.0, 1.0, 0.0),
    ///     Point3::new(0.0, 1.0, 0.0),
    ///     Point3::new(0.0, 1.0, 1.0),
    /// ];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let pt = ParametricCurve::subs(&bspcurve, 1.2);
    /// let t = bspcurve.search_nearest_parameter(pt, Some(0.8), 100).unwrap();
    /// assert_near!(t, 1.2);
    /// ```
    /// # Remarks
    /// It may converge to a local solution depending on the hint.
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::from(
    ///     vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 3.0]
    /// );
    /// let ctrl_pts = vec![
    ///     Point3::new(0.0, 0.0, 0.0),
    ///     Point3::new(1.0, 0.0, 0.0),
    ///     Point3::new(1.0, 1.0, 0.0),
    ///     Point3::new(0.0, 1.0, 0.0),
    ///     Point3::new(0.0, 1.0, 1.0),
    /// ];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let pt = Point3::new(0.0, 0.5, 1.0);
    /// let t = bspcurve.search_nearest_parameter(pt, Some(0.8), 100).unwrap();
    /// let pt0 = ParametricCurve::subs(&bspcurve, t);
    /// let pt1 = ParametricCurve::subs(&bspcurve, 3.0);
    /// // the point corresponding the obtained parameter is not
    /// // the globally nearest point in the curve.
    /// assert!((pt0 - pt).magnitude() > (pt1 - pt).magnitude());
    /// ```
    #[inline(always)]
    fn search_nearest_parameter(&self, point: P, hint: Option<f64>, trial: usize) -> Option<f64> {
        let hint = match hint {
            Some(hint) => hint,
            None => algo::curve::presearch(self, point, self.parameter_range(), PRESEARCH_DIVISION),
        };
        algo::curve::search_nearest_parameter(self, point, hint, trial)
    }
}
 
impl<P> SearchParameter for BSplineCurve<P>
where
    P: ControlPoint<f64>
        + EuclideanSpace<Scalar = f64, Diff = <P as ControlPoint<f64>>::Diff>
        + MetricSpace<Metric = f64>,
    <P as ControlPoint<f64>>::Diff: InnerSpace<Scalar = f64> + Tolerance,
{
    type Point = P;
    type Parameter = f64;
    #[inline(always)]
    fn search_parameter(&self, point: P, hint: Option<f64>, trial: usize) -> Option<f64> {
        let hint = match hint {
            Some(hint) => hint,
            None => algo::curve::presearch(self, point, self.parameter_range(), PRESEARCH_DIVISION),
        };
        algo::curve::search_parameter(self, point, hint, trial)
    }
}

impl<P> BSplineCurve<P>
where P: MetricSpace<Metric = f64> + Index<usize, Output = f64> + Bounded<f64> + Copy
{
    /// Returns the bounding box including all control points.
    #[inline(always)]
    pub fn roughly_bounding_box(&self) -> BoundingBox<P> { self.control_points.iter().collect() }
}

impl<P: Clone> Invertible for BSplineCurve<P> {
    #[inline(always)]
    fn invert(&mut self) { self.invert(); }
}

impl<M, P> Transformed<M> for BSplineCurve<P>
where
    P: EuclideanSpace,
    M: Transform<P>,
{
    #[inline(always)]
    fn transform_by(&mut self, trans: M) {
        self.control_points
            .iter_mut()
            .for_each(|pt| *pt = trans.transform_point(*pt))
    }
}

#[test]
fn test_near_as_curve() {
    let knot_vec = KnotVec::from(vec![
        0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 5.0, 5.0, 5.0,
    ]);
    let control_points = vec![
        Vector4::new(1.0, 0.0, 0.0, 0.0),
        Vector4::new(0.0, 1.0, 0.0, 0.0),
        Vector4::new(0.0, 0.0, 1.0, 0.0),
        Vector4::new(0.0, 0.0, 0.0, 1.0),
        Vector4::new(1.0, 1.0, 0.0, 0.0),
        Vector4::new(1.0, 0.0, 1.0, 0.0),
        Vector4::new(1.0, 0.0, 0.0, 1.0),
        Vector4::new(1.0, 1.0, 1.0, 0.0),
    ];
    let bspline0 = BSplineCurve::new(knot_vec, control_points.clone());
    let knot_vec = KnotVec::from(vec![
        0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 2.5, 3.0, 4.0, 5.0, 5.0, 5.0, 5.0,
    ]);
    let control_points = vec![
        control_points[0].clone(),
        control_points[1].clone(),
        control_points[2].clone(),
        &control_points[3] * (5.0 / 6.0) + &control_points[2] * (1.0 / 6.0),
        &control_points[4] * 0.5 + &control_points[3] * 0.5,
        &control_points[5] * (1.0 / 6.0) + &control_points[4] * (5.0 / 6.0),
        control_points[5].clone(),
        control_points[6].clone(),
        control_points[7].clone(),
    ];
    let bspline1 = BSplineCurve::new(knot_vec, control_points.clone());
    let knot_vec = KnotVec::from(vec![
        0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 5.0, 5.0, 5.0,
    ]);
    let control_points = vec![
        Vector4::new(1.0, 0.0, 0.0, 0.0),
        Vector4::new(0.0, 1.0, 0.0, 0.0),
        Vector4::new(0.0, 0.0, 1.0, 0.0),
        Vector4::new(0.0, 0.0, 0.0, 1.0),
        Vector4::new(1.0, 1.01, 0.0, 0.0),
        Vector4::new(1.0, 0.0, 1.0, 0.0),
        Vector4::new(1.0, 0.0, 0.0, 1.0),
        Vector4::new(1.0, 1.0, 1.0, 0.0),
    ];
    let bspline2 = BSplineCurve::new(knot_vec, control_points.clone());
    assert!(bspline0.near_as_curve(&bspline1));
    assert!(!bspline0.near_as_curve(&bspline2));
}

#[test]
fn test_parameter_division() {
    let knot_vec = KnotVec::uniform_knot(2, 3);
    let ctrl_pts = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
    ];
    let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    let tol = 0.01;
    let (div, pts) = bspcurve.parameter_division(bspcurve.parameter_range(), tol);
    let knot_vec = bspcurve.knot_vec();
    assert_eq!(knot_vec[0], div[0]);
    assert_eq!(knot_vec.range_length(), div.last().unwrap() - div[0]);
    for i in 1..div.len() {
        let pt0 = bspcurve.subs(div[i - 1]);
        assert_eq!(pt0, pts[i - 1]);
        let pt1 = bspcurve.subs(div[i]);
        assert_eq!(pt1, pts[i]);
        let value_middle = pt0 + (pt1 - pt0) / 2.0;
        let param_middle = bspcurve.subs((div[i - 1] + div[i]) / 2.0);
        assert!(value_middle.distance(param_middle) < tol);
    }
}
