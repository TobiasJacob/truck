#[macro_export]
macro_rules! impl_curve {
    ($mod: tt, $impl_curve_mod: ident) => {
        mod $impl_curve_mod {
            use super::$mod;
            use std::convert::{TryFrom, TryInto};
            use std::result::Result;
            use $crate::alias::*;
            $crate::sub_impl_curve!($mod, Point2, Vector2, Matrix3, Vector3);
            $crate::sub_impl_curve!($mod, Point3, Vector3, Matrix4, Vector4);
            fn to_parameter<'a, C>(trim: &'a [$mod::TrimmingSelect], curve: &C) -> Option<f64>
            where
                C: SearchParameter<Parameter = f64>,
                C::Point: From<&'a $mod::CartesianPoint>, {
                match trim.len() {
                    1 => match &trim[0] {
                        $mod::TrimmingSelect::ParameterValue(x) => Some(x.0),
                        $mod::TrimmingSelect::CartesianPoint(p) => {
                            let p = C::Point::from(p);
                            curve.search_parameter(p, None, 100)
                        }
                    },
                    2 => {
                        if let $mod::TrimmingSelect::ParameterValue(x) = &trim[0] {
                            Some(x.0)
                        } else if let $mod::TrimmingSelect::ParameterValue(x) = &trim[1] {
                            Some(x.0)
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            $crate::impl_try_from!{
                impl TryFrom<&$mod::Pcurve> for PCurve {
                    fn try_from(curve: &$mod::Pcurve) -> Result<Self, ExpressParseError> {
                        let surface = Surface::try_from(&curve.basis_surface)?;
                        let curve = if let Some($mod::RepresentationItemAny::GeometricRepresentationItem(item)) = curve.reference_to_curve.representation.items.first() {
                            if let $mod::GeometricRepresentationItemAny::Curve(curve) = item.as_ref() {
                                Curve::try_from(&**curve)?
                            } else {
                                return Err("The references curve is not a curve.".to_string());
                            }
                        } else {
                            return Err("The references curve is not a curve.".to_string());
                        };
                        Ok(PCurve::new(Box::new(curve), Box::new(surface)))
                    }
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! pcurve_converting {
    (Point2) => {
        Err("This is not 2-dim curve.".to_string())
    };
    (Point3) => {
        Ok(Self::PCurve(curve.try_from()?))
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! sub_impl_curve {
    ($mod: tt, $point: tt, $vector: tt, $matrix: tt, $homogeneous: tt) => {
        $crate::impl_from!(
            impl From<&$mod::Line> for Line<$point> {
                fn from(line: &$mod::Line) -> Self {
                    let p = <$point>::from(&line.pnt);
                    let q = p + <$vector>::from(&line.dir);
                    Self(p, q)
                }
            }
            impl From<&$mod::Polyline> for PolylineCurve<$point> {
                fn from(poly: &$mod::Polyline) -> Self {
                    Self(poly.points.iter().map(|pt| <$point>::from(pt)).collect())
                }
            }
            impl From<&$mod::BSplineCurveWithKnots> for BSplineCurve<$point> {
                fn from(curve: &$mod::BSplineCurveWithKnots) -> Self {
                    let knots = curve.knots.iter().map(|a| **a).collect();
                    let multi = curve
                        .knot_multiplicities
                        .iter()
                        .map(|n| *n as usize)
                        .collect();
                    let knots = KnotVec::from_single_multi(knots, multi).unwrap();
                    let ctrpts = curve
                        .b_spline_curve
                        .control_points_list
                        .iter()
                        .map(Into::into)
                        .collect();
                    Self::new(knots, ctrpts)
                }
            }
            impl From<&$mod::BezierCurve> for BSplineCurve<$point> {
                fn from(curve: &$mod::BezierCurve) -> Self {
                    let curve = &curve.b_spline_curve;
                    let degree = curve.degree as usize;
                    let knots = KnotVec::bezier_knot(degree);
                    let ctrpts = curve.control_points_list.iter().map(Into::into).collect();
                    Self::new(knots, ctrpts)
                }
            }
            impl From<&$mod::QuasiUniformCurve> for BSplineCurve<$point> {
                fn from(curve: &$mod::QuasiUniformCurve) -> Self {
                    let curve = &curve.b_spline_curve;
                    let num_ctrl = curve.control_points_list.len();
                    let degree = curve.degree as usize;
                    let division = num_ctrl + 2 - degree;
                    let mut knots = KnotVec::uniform_knot(degree, division);
                    knots.transform(division as f64, 0.0);
                    let ctrpts = curve.control_points_list.iter().map(Into::into).collect();
                    Self::new(knots, ctrpts)
                }
            }
            impl From<&$mod::RationalBSplineCurve> for NURBSCurve<$homogeneous> {
                fn from(curve: &$mod::RationalBSplineCurve) -> Self {
                    let bcurve = &curve.b_spline_curve;
                    let degree = bcurve.degree as usize;
                    let knots = KnotVec::bezier_knot(degree);
                    let ctrpts = bcurve
                        .control_points_list
                        .iter()
                        .zip(&curve.weights_data)
                        .map(|(pt, w)| <$homogeneous>::from_point_weight(pt.into(), *w))
                        .collect();
                    Self::new(BSplineCurve::new(knots, ctrpts))
                }
            }
            impl From<&$mod::UniformCurve> for BSplineCurve<$point> {
                fn from(curve: &$mod::UniformCurve) -> Self {
                    let curve = &curve.b_spline_curve;
                    let num_ctrl = curve.control_points_list.len();
                    let degree = curve.degree as usize;
                    let knots = KnotVec::try_from(
                        (0..degree + num_ctrl + 1)
                            .map(|i| i as f64 - degree as f64)
                            .collect::<Vec<_>>(),
                    );
                    let ctrpts = curve.control_points_list.iter().map(Into::into).collect();
                    Self::new(knots.unwrap(), ctrpts)
                }
            }
        );
        $crate::impl_try_from!(
            impl TryFrom<&$mod::Circle> for Ellipse<$point, $matrix> {
                fn try_from(circle: &$mod::Circle) -> Result<Self, ExpressParseError> {
                    let radius: f64 = **circle.radius;
                    let transform =
                        <$matrix>::try_from(&circle.conic.position)? * <$matrix>::from_scale(radius);
                    Ok(Processor::new(UnitCircle::new()).transformed(transform))
                }
            }
            impl TryFrom<&$mod::Ellipse> for Ellipse<$point, $matrix> {
                fn try_from(circle: &$mod::Ellipse) -> Result<Self, ExpressParseError> {
                    let radius0: f64 = **circle.semi_axis_1;
                    let radius1: f64 = **circle.semi_axis_2;
                    let transform = <$matrix>::try_from(&circle.conic.position)?
                        * <$matrix>::from(Matrix3::from_nonuniform_scale(radius0, radius1));
                    Ok(Processor::new(UnitCircle::new()).transformed(transform))
                }
            }
            impl TryFrom<&$mod::Hyperbola> for Hyperbola<$point, $matrix> {
                fn try_from(circle: &$mod::Hyperbola) -> Result<Self, ExpressParseError> {
                    let radius0: f64 = **circle.semi_axis;
                    let radius1: f64 = **circle.semi_imag_axis;
                    let transform = <$matrix>::try_from(&circle.conic.position)?
                        * <$matrix>::from(Matrix3::from_nonuniform_scale(radius0, radius1));
                    Ok(Processor::new(UnitHyperbola::new()).transformed(transform))
                }
            }
            impl TryFrom<&$mod::Parabola> for Parabola<$point, $matrix> {
                fn try_from(circle: &$mod::Parabola) -> Result<Self, ExpressParseError> {
                    let f: f64 = *circle.focal_dist;
                    let transform =
                        <$matrix>::try_from(&circle.conic.position)? * <$matrix>::from_scale(f);
                    Ok(Processor::new(UnitParabola::new()).transformed(transform))
                }
            }
            impl TryFrom<&$mod::ConicAny> for Conic<$point, $matrix> {
                fn try_from(conic: &$mod::ConicAny) -> Result<Self, ExpressParseError> {
                    use $mod::ConicAny::*;
                    match conic {
                        Conic(_) => Err("not enough data!".to_string()),
                        Circle(c) => Ok(Self::Ellipse((&**c).try_into()?)),
                        Ellipse(c) => Ok(Self::Ellipse((&**c).try_into()?)),
                        Hyperbola(c) => Ok(Self::Hyperbola((&**c).try_into()?)),
                        Parabola(c) => Ok(Self::Parabola((&**c).try_into()?)),
                    }
                }
            }
            impl TryFrom<&$mod::BSplineCurveAny> for NURBSCurve<$homogeneous> {
                fn try_from(curve: &$mod::BSplineCurveAny) -> Result<Self, ExpressParseError> {
                    use $mod::BSplineCurveAny as BSCA;
                    match curve {
                        BSCA::BSplineCurve(_) => Err("not enough data!".to_string()),
                        BSCA::BSplineCurveWithKnots(x) => Ok(NURBSCurve::new(BSplineCurve::lift_up(
                            BSplineCurve::<$point>::from(&**x),
                        ))),
                        BSCA::UniformCurve(x) => Ok(NURBSCurve::new(BSplineCurve::lift_up(
                            BSplineCurve::<$point>::from(&**x),
                        ))),
                        BSCA::QuasiUniformCurve(x) => Ok(NURBSCurve::new(BSplineCurve::lift_up(
                            BSplineCurve::<$point>::from(&**x),
                        ))),
                        BSCA::BezierCurve(x) => Ok(NURBSCurve::new(BSplineCurve::lift_up(
                            BSplineCurve::<$point>::from(&**x),
                        ))),
                        BSCA::RationalBSplineCurve(x) => Ok(NURBSCurve::from(&**x)),
                    }
                }
            }
            impl TryFrom<&$mod::TrimmedCurve> for TrimmedCurve<Box<Curve<$point, $homogeneous, $matrix>>> {
                fn try_from(curve: &$mod::TrimmedCurve) -> Result<Self, ExpressParseError> {
                    let basis_curve: Curve<$point, $homogeneous, $matrix> = TryFrom::try_from(&curve.basis_curve)?;
                    let t0 = match to_parameter(&curve.trim_1, &basis_curve) {
                        Some(t) => t,
                        None => return Err(format!("invalid trimming select: {:?}", curve.trim_1)),
                    };
                    let t1 = match to_parameter(&curve.trim_2, &basis_curve) {
                        Some(t) => t,
                        None => return Err(format!("invalid trimming select: {:?}", curve.trim_2)),
                    };
                    Ok(TrimmedCurve::new(Box::new(basis_curve), (t0, t1)))
                }
            }
            impl TryFrom<&$mod::BoundedCurveAny> for Curve<$point, $homogeneous, $matrix> {
                fn try_from(curve: &$mod::BoundedCurveAny) -> Result<Self, ExpressParseError> {
                    use $mod::BoundedCurveAny as BCA;
                    match curve {
                        BCA::BSplineCurve(x) => Ok(Curve::NURBSCurve((&**x).try_into()?)),
                        BCA::TrimmedCurve(x) => Ok(Curve::TrimmedCurve((&**x).try_into()?)),
                        _ => Err("unimplemented!".to_string()),
                    }
                }
            }
            impl TryFrom<&$mod::CurveAny> for Curve<$point, $homogeneous, $matrix> {
                fn try_from(curve: &$mod::CurveAny) -> Result<Self, ExpressParseError> {
                    use $mod::CurveAny::*;
                    match curve {
                        Curve(_) => Err("not enough data!".to_string()),
                        Line(x) => Ok(Self::Line((&**x).into())),
                        Conic(x) => Ok(Self::Conic((&**x).try_into()?)),
                        BoundedCurve(x) => (&**x).try_into(),
                        Pcurve(x) => $crate::pcurve_converting!($point),
                        _ => Err("unimplemented!".to_string()),
                    }
                }
            }
        );
        impl From<Line<$point>> for $mod::Line {
            fn from(line: Line<$point>) -> Self {
                Self::new(Empty::empty(), line.0.into(), (line.1 - line.0).into())
            }
        }
    };
}
