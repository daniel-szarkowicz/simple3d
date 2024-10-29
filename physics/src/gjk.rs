use std::ops::{Add, Mul, Sub};

use clipper2::{FillRule, Paths};
use nalgebra::{AbstractRotation, Const, DimMin, Matrix, Rotation3};
// use smallvec::SmallVec;

use crate::{Float, Vec3};

// type SimplexData = SmallVec<[SupportPoint; 4]>;
type SimplexData = Vec<SupportPoint>;

const TOLERANCE: Float = 1e-7;
const SIMPLEX_MAX_DIM: usize = 4;
const EPA_MAX_ITER: usize = 10;
const GJK_MAX_ITER: usize = 12;

pub enum Feature {
    Point(Vec3),
    Segment(Vec3, Vec3),
    Polygon { normal: Vec3, points: Box<[Vec3]> },
}

pub trait Support {
    fn support(&self, direction: &Vec3) -> Vec3;
    fn radius(&self) -> f64;
    fn base(&self) -> Vec3;
    fn feature(&self, direction: &Vec3) -> Feature;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SupportPoint {
    pub diff: Vec3,
    pub a: Vec3,
}

impl SupportPoint {
    pub fn new(a: &impl Support, b: &impl Support, dir: &Vec3) -> Self {
        let a = a.support(dir);
        let b = b.support(&-dir);
        Self { diff: a - b, a }
    }
}

// currently gjk always returns a single point
// to simulate resting contact we need more points
// we first have to know all types of contact that can happen
// gjk uses simplexes for calculations different simplexes can represent
// different contact types:
// 1-simplex: this represents a single point, like two spheres colliding
// 2-simplex: this either represents two parallel edges, or an edge and a point
//            if it's two edges, then the support points are different
//            if it's an edge and a point, then one of the support points is the same
//              in this case only one contact should be returned (closest point)
// 3-simplex: case vertex-face: return closest
//            case edge-edge (not parallel): return closest
//            case edge-face: return the single edge point and the closest to the double edge point
//            case face-face: return all three
fn handle_return(
    a: &impl Support,
    b: &impl Support,
    s: &SimplexData,
    p: &SupportPoint,
) -> Option<(Vec3, Vec3, Vec3)> {
    closest_point_to_contact(a, b, p)
    // match s.len() {
    //     1 => closest_point_to_contact(a, b, p).into_iter().collect(),
    //     2 => handle_return_2(a, b, &s[0], &s[1], p),
    //     3 => handle_return_3(a, b, &s[0], &s[1], &s[2], p),
    //     _ => unreachable!(),
    // }
}

fn handle_return_2(
    a: &impl Support,
    b: &impl Support,
    s1: &SupportPoint,
    s2: &SupportPoint,
    p: &SupportPoint,
) -> Vec<(Vec3, Vec3, Vec3)> {
    if (s1.a - s2.a).magnitude_squared() < TOLERANCE {
        // the point from `a` is the same, this is a vertex-edge collision
        return closest_point_to_contact(a, b, p).into_iter().collect();
    }
    let s1_b = s1.a - s1.diff;
    let s2_b = s2.a - s2.diff;
    if (s1_b - s2_b).magnitude_squared() < TOLERANCE {
        // the point from `b` is the same, this is a vertex-edge collision
        return closest_point_to_contact(a, b, p).into_iter().collect();
    }
    // we are in an edge-edge collision
    println!("TODO: edge-edge collision");
    closest_point_to_contact(a, b, p).into_iter().collect()
}

fn handle_return_3(
    a: &impl Support,
    b: &impl Support,
    s1: &SupportPoint,
    s2: &SupportPoint,
    s3: &SupportPoint,
    p: &SupportPoint,
) -> Vec<(Vec3, Vec3, Vec3)> {
    let s12_a = (s1.a - s2.a).magnitude_squared() < TOLERANCE;
    let s13_a = (s1.a - s3.a).magnitude_squared() < TOLERANCE;
    let s23_a = (s2.a - s3.a).magnitude_squared() < TOLERANCE;
    if s12_a && s13_a && s23_a {
        // the point from `a` is the same, this is a vertex-face collision
        return closest_point_to_contact(a, b, p).into_iter().collect();
    }
    debug_assert!(!(s12_a && s13_a));
    debug_assert!(!(s13_a && s23_a));
    debug_assert!(!(s12_a && s23_a));
    if s12_a || s13_a || s23_a {
        // two points from `a` are the same, this is an edge-? collision
        println!("TODO: handle a edge-? collision");
        return closest_point_to_contact(a, b, p).into_iter().collect();
    }
    let s1_b = s1.a - s1.diff;
    let s2_b = s2.a - s2.diff;
    let s3_b = s3.a - s3.diff;
    let s12_b = (s1_b - s2_b).magnitude_squared() < TOLERANCE;
    let s13_b = (s1_b - s3_b).magnitude_squared() < TOLERANCE;
    let s23_b = (s2_b - s3_b).magnitude_squared() < TOLERANCE;
    if s12_b && s13_b && s23_b {
        // the point from `b` is the same, this is a vertex-face collision
        return closest_point_to_contact(a, b, p).into_iter().collect();
    }
    debug_assert!(!(s12_b && s13_b));
    debug_assert!(!(s13_b && s23_b));
    debug_assert!(!(s12_b && s23_b));
    if s12_b || s13_b || s23_b {
        // two points from `b` are the same, this is an edge-? collision
        println!("todo: handle b edge-? collision");
        return closest_point_to_contact(a, b, p).into_iter().collect();
    }
    println!("todo: handle face-face collision");
    closest_point_to_contact(a, b, p).into_iter().collect()
    // let result: Vec<_> = closest_point_to_contact(a, b, s1)
    //     .into_iter()
    //     .chain(closest_point_to_contact(a, b, s2))
    //     .chain(closest_point_to_contact(a, b, s3))
    //     .collect();
    // println!("handle-3: {}", result.len());
    // result
}

fn project(p: &Vec3, sn: &Vec3, sp: &Vec3) -> Vec3 {
    p + (sp - p).dot(sn) * sn
}

/*
inverse projection
    p' = p_0 + l * n_0, (p' - p_1) dot n_1 = 0
    (p_0 + l * n_0 - p_1) dot n_1 = 0
    l * n_0_para = n_1 dot (p_0 - p_1)
*/

fn project_along(
    p: &Vec3,
    along: &Vec3,
    surf_norm: &Vec3,
    surf_p: &Vec3,
) -> Vec3 {
    let surf_dist = (p - surf_p).dot(surf_norm);
    let along_ratio = along.dot(surf_norm);
    let l = surf_dist / along_ratio;
    p - l * along
}

pub fn get_contacts(
    a: &impl Support,
    b: &impl Support,
) -> Vec<(Vec3, Vec3, Vec3)> {
    let Some((p1, p2, n)) = gjk(a, b) else {
        return vec![];
    };
    match (a.feature(&-n), b.feature(&n)) {
        (Feature::Point(_), _) | (_, Feature::Point(_)) => vec![(p1, p2, n)],
        (Feature::Segment(_, _), Feature::Segment(_, _)) => vec![(p1, p2, n)],
        (Feature::Segment(_, _), _) | (_, Feature::Segment(_, _)) => {
            todo!("segment needs to be clipped to the polygon")
        }
        (
            Feature::Polygon {
                normal: n1,
                points: p1s,
            },
            Feature::Polygon {
                normal: n2,
                points: p2s,
            },
        ) => {
            // debug_assert!(n1.dot(&-n) > 0.0);
            // debug_assert!(n2.dot(&n) > 0.0);
            let depth = n.dot(&(p2 - p1));
            let rot = Rotation3::rotation_between(&n, &Vec3::z()).unwrap();
            let np1s: Vec<_> = p1s
                .iter()
                .map(|p| rot * project(p, &n, &Vec3::zeros()))
                .map(|p| [p.x, p.y])
                .collect();
            let np2s: Vec<_> = p2s
                .iter()
                .map(|p| rot * project(p, &n, &Vec3::zeros()))
                .map(|p| [p.x, p.y])
                .collect();
            let poly1: Paths = np1s.into();
            let poly2: Paths = np2s.into();
            let intersection =
                clipper2::intersect(poly1, poly2, FillRule::default()).unwrap();
            let rot_inv = rot.inverse();
            intersection
                .into_iter()
                .flatten()
                .map(|p| rot_inv * Vec3::new(p.x(), p.y(), 0.0))
                .map(|p| {
                    (
                        project_along(&p, &n, &n1, &p1s[0]),
                        project_along(&p, &n, &n2, &p2s[0]),
                    )
                })
                .filter(|(p1, p2)| n.dot(&(p1 - p2)) <= depth)
                .map(|(p1, p2)| (p1, p2, n))
                .chain(std::iter::once((p1, p2, n)))
                .collect()
        }
    }
}

pub fn gjk(a: &impl Support, b: &impl Support) -> Option<(Vec3, Vec3, Vec3)> {
    let mut s = SimplexData::with_capacity(4);
    s.push(SupportPoint::new(a, b, &(a.base() - b.base())));
    let mut prev_dist = f64::INFINITY;
    let mut closest_point = closest_simplex(&mut s);
    let mut dist_diff = 0.0;
    for _ in 0..GJK_MAX_ITER {
        let dist = closest_point.diff.magnitude();
        // dbg!(&s);
        // dbg!(closest_point.diff);
        if s.len() == SIMPLEX_MAX_DIM {
            // return epa(a, b, s.into_vec());
            // println!("going epa");
            return epa(a, b, s);
        }
        debug_assert!(
            dist <= prev_dist + TOLERANCE,
            "prev_dist={prev_dist}, dist={dist}",
        );
        dist_diff = prev_dist - dist;
        if prev_dist - dist <= TOLERANCE {
            // println!("1: {}", s.len());
            return handle_return(a, b, &s, &closest_point);
        }
        prev_dist = dist;
        let new_point = SupportPoint::new(a, b, &-closest_point.diff);
        if closest_point
            .diff
            .dot(&(new_point.diff - closest_point.diff))
            >= -TOLERANCE
        {
            // println!("2: {}", s.len());
            return handle_return(a, b, &s, &closest_point);
        }
        s.push(new_point);
        closest_point = closest_simplex(&mut s);
    }
    eprintln!(
        "gjk didn't converge in {GJK_MAX_ITER} steps \
        (dist = {prev_dist:0.10}, diff = {dist_diff:0.10})"
    );
    // println!("3: {}", s.len());
    handle_return(a, b, &s, &closest_point)
}

#[allow(clippy::similar_names)]
#[allow(clippy::too_many_lines)]
fn best_simplex(s: &mut SimplexData) {
    match s.len() {
        1 => {}
        2 => {
            let ab = s[1].diff - s[0].diff;
            if ab.dot(&-s[0].diff) < 0.0 {
                s.remove(1);
                return;
            }
            if ab.dot(&-s[1].diff) > 0.0 {
                s.remove(0);
            }
        }
        3 => {
            // háromszög síkjára merőleges
            let abc_perp =
                (s[1].diff - s[0].diff).cross(&(s[2].diff - s[0].diff));

            // háromszögből kifele mutat, ac-re merőleges
            let ac_perp = abc_perp.cross(&(s[2].diff - s[0].diff));
            // ha az origó egy irányba van az oldal normáljával
            let ac = ac_perp.dot(&-s[2].diff) > 0.0;
            // háromszögből kifele mutat, bc-re merőleges
            let bc_perp = (s[2].diff - s[1].diff).cross(&abc_perp);
            // ha az origó egy irányba van az oldal normáljával
            let bc = bc_perp.dot(&-s[2].diff) > 0.0;
            let ab_perp = (s[1].diff - s[0].diff).cross(&abc_perp);
            let ab = ab_perp.dot(&-s[1].diff) > 0.0;

            match (ab, bc, ac) {
                (true, true, true) => unreachable!(
                    "point cannot be on all three sides of a triangle"
                ),
                (true, true, false) => {
                    triangle_two_sides_subcheck(s);
                }
                (false, true, true) => {
                    s.rotate_left(1);
                    triangle_two_sides_subcheck(s);
                }
                (true, false, true) => {
                    s.rotate_left(2);
                    triangle_two_sides_subcheck(s);
                }
                (true, false, false) => {
                    s.remove(2);
                    best_simplex(s);
                }
                (false, true, false) => {
                    s.remove(0);
                    best_simplex(s);
                }
                (false, false, true) => {
                    s.remove(1);
                    best_simplex(s);
                }
                (false, false, false) => {
                    if abc_perp.dot(&-s[2].diff) <= 0.0 {
                        s.reverse();
                    }
                }
            }
        }
        4 => {
            let abd_perp =
                (s[1].diff - s[0].diff).cross(&(s[3].diff - s[0].diff));
            let bcd_perp =
                (s[2].diff - s[1].diff).cross(&(s[3].diff - s[1].diff));
            let cad_perp =
                (s[0].diff - s[2].diff).cross(&(s[3].diff - s[2].diff));
            debug_assert!(
                abd_perp.dot(&(s[2].diff - s[3].diff)) < 0.0,
                "abd={abd_perp:?}"
            );
            debug_assert!(
                bcd_perp.dot(&(s[0].diff - s[3].diff)) < 0.0,
                "bcd={bcd_perp:?}"
            );
            debug_assert!(
                cad_perp.dot(&(s[1].diff - s[3].diff)) < 0.0,
                "cad={cad_perp:?}"
            );
            let abd = abd_perp.dot(&-s[3].diff) > 0.0;
            let bcd = bcd_perp.dot(&-s[3].diff) > 0.0;
            let cad = cad_perp.dot(&-s[3].diff) > 0.0;

            match (abd, bcd, cad) {
                (true, true, true) => {
                    tetrahedron_three_sides_subcheck(
                        s, abd_perp, bcd_perp, cad_perp,
                    );
                }
                (true, true, false) => {
                    tetrahedron_two_sides_subcheck(s, abd_perp, bcd_perp);
                }
                (false, true, true) => {
                    s[0..3].rotate_left(1);
                    tetrahedron_two_sides_subcheck(s, bcd_perp, cad_perp);
                }
                (true, false, true) => {
                    s[0..3].rotate_left(2);
                    tetrahedron_two_sides_subcheck(s, cad_perp, abd_perp);
                }
                (true, false, false) => {
                    s.remove(2);
                    tetrahedron_triangle_subcheck(s, abd_perp);
                }
                (false, true, false) => {
                    s[0..3].rotate_left(1);
                    s.remove(2);
                    tetrahedron_triangle_subcheck(s, bcd_perp);
                }
                (false, false, true) => {
                    s[0..3].rotate_left(2);
                    s.remove(2);
                    tetrahedron_triangle_subcheck(s, cad_perp);
                }
                (false, false, false) => {}
            }
        }
        _ => unreachable!(),
    }
}

// the edges with index 0, 1 and 1, 2 both have the origin above them
// the shared vertex is 1
fn triangle_two_sides_subcheck(s: &mut SimplexData) {
    let edgevec1 = s[1].diff - s[0].diff;
    if edgevec1.dot(&-s[1].diff) > 0.0 {
        s.remove(0);
        return best_simplex(s);
    }
    s.remove(2);
    best_simplex(s);
}

// the faces with index 0, 1, 3 and 1, 2, 3 both have the origin above them
// the shared edge is 1, 3
fn tetrahedron_two_sides_subcheck(
    s: &mut SimplexData,
    perp1: Vec3,
    perp2: Vec3,
) {
    let out1_1 = (s[3].diff - s[1].diff).cross(&perp1);
    let out1_2 = perp1.cross(&(s[3].diff - s[0].diff));
    debug_assert!(out1_1.dot(&(s[3].diff - s[0].diff)) > 0.0);
    debug_assert!(out1_2.dot(&(s[3].diff - s[1].diff)) > 0.0);

    let out2_1 = perp2.cross(&(s[3].diff - s[1].diff));
    let out2_2 = (s[3].diff - s[2].diff).cross(&perp2);
    debug_assert!(out2_1.dot(&(s[3].diff - s[2].diff)) > 0.0);
    debug_assert!(out2_2.dot(&(s[3].diff - s[1].diff)) > 0.0);

    let c1_1 = out1_1.dot(&-s[3].diff) < 0.0;
    let c1_2 = out1_2.dot(&-s[3].diff) < 0.0;
    let c2_1 = out2_1.dot(&-s[3].diff) < 0.0;
    let c2_2 = out2_2.dot(&-s[3].diff) < 0.0;

    if c1_1 && c1_2 {
        // it is inside the first face
        s.remove(2);
        return;
    }

    if c2_1 && c2_2 {
        // it is inside the second face
        s.remove(0);
        return;
    }

    // it is on one of the edges
    let e1_1 = -s[0].diff.dot(&(s[0].diff - s[3].diff)) < 0.0;
    let e1_2 = -s[3].diff.dot(&(s[3].diff - s[0].diff)) < 0.0;

    let e2_1 = -s[1].diff.dot(&(s[1].diff - s[3].diff)) < 0.0;
    let e2_2 = -s[3].diff.dot(&(s[3].diff - s[1].diff)) < 0.0;

    let e3_1 = -s[2].diff.dot(&(s[2].diff - s[3].diff)) < 0.0;
    let e3_2 = -s[3].diff.dot(&(s[3].diff - s[2].diff)) < 0.0;

    if e1_1 && e1_2 && !c1_2 {
        s.remove(2);
        s.remove(1);
        return;
    }

    if e2_1 && e2_2 && !c1_1 && !c2_1 {
        s.remove(2);
        s.remove(0);
        return;
    }

    if e3_1 && e3_2 && !c2_2 {
        s.remove(1);
        s.remove(0);
        return;
    }

    // it is not on the edges, it must be the new point
    s.remove(2);
    s.remove(1);
    s.remove(0);
}

// all three faces have the origin above them
fn tetrahedron_three_sides_subcheck(
    s: &mut SimplexData,
    perp1: Vec3,
    perp2: Vec3,
    perp3: Vec3,
) {
    let out1_1 = (s[3].diff - s[1].diff).cross(&perp1);
    let out1_2 = perp1.cross(&(s[3].diff - s[0].diff));
    debug_assert!(out1_1.dot(&(s[3].diff - s[0].diff)) > 0.0);
    debug_assert!(out1_2.dot(&(s[3].diff - s[1].diff)) > 0.0);

    let out2_1 = perp2.cross(&(s[3].diff - s[1].diff));
    let out2_2 = (s[3].diff - s[2].diff).cross(&perp2);
    debug_assert!(out2_1.dot(&(s[3].diff - s[2].diff)) > 0.0);
    debug_assert!(out2_2.dot(&(s[3].diff - s[1].diff)) > 0.0);

    let out3_1 = perp3.cross(&(s[3].diff - s[2].diff));
    let out3_2 = (s[3].diff - s[0].diff).cross(&perp3);
    debug_assert!(out3_1.dot(&(s[3].diff - s[0].diff)) > 0.0);
    debug_assert!(out3_2.dot(&(s[3].diff - s[2].diff)) > 0.0);

    let c1_1 = out1_1.dot(&-s[3].diff) < 0.0;
    let c1_2 = out1_2.dot(&-s[3].diff) < 0.0;
    let c2_1 = out2_1.dot(&-s[3].diff) < 0.0;
    let c2_2 = out2_2.dot(&-s[3].diff) < 0.0;
    let c3_1 = out3_1.dot(&-s[3].diff) < 0.0;
    let c3_2 = out3_2.dot(&-s[3].diff) < 0.0;

    // eprintln!("c1_1 = {c1_1}, c1_2 = {c1_2}, c2_1 = {c2_1}, c2_2 = {c2_2}, c3_1 = {c3_1}, c3_2 = {c3_2}");

    // dbg!(&s);
    if c1_1 && c1_2 {
        // it is inside the first face
        s.remove(2);
        // return tetrahedron_triangle_subcheck(s, perp1);
        return;
    }

    if c2_1 && c2_2 {
        // it is inside the second face
        s.remove(0);
        return;
    }

    if c3_1 && c3_2 {
        s.remove(1);
        // the winding order of the triangle has to be fixed
        s.reverse();
        return;
    }

    // it is on one of the edges
    let e1_1 = -s[0].diff.dot(&(s[0].diff - s[3].diff)) < 0.0;
    let e1_2 = -s[3].diff.dot(&(s[3].diff - s[0].diff)) < 0.0;

    let e2_1 = -s[1].diff.dot(&(s[1].diff - s[3].diff)) < 0.0;
    let e2_2 = -s[3].diff.dot(&(s[3].diff - s[1].diff)) < 0.0;

    let e3_1 = -s[2].diff.dot(&(s[2].diff - s[3].diff)) < 0.0;
    let e3_2 = -s[3].diff.dot(&(s[3].diff - s[2].diff)) < 0.0;

    if e1_1 && e1_2 && !c1_2 && !c3_2 {
        s.remove(2);
        s.remove(1);
        return;
    }

    if e2_1 && e2_2 && !c1_1 && !c2_1 {
        s.remove(2);
        s.remove(0);
        return;
    }

    if e3_1 && e3_2 && !c2_2 && !c3_1 {
        s.remove(1);
        s.remove(0);
        return;
    }

    // it is not on the edges, it must be the new point
    s.remove(2);
    s.remove(1);
    s.remove(0);
}

#[allow(clippy::similar_names)]
fn tetrahedron_triangle_subcheck(s: &mut SimplexData, xyd_perp: Vec3) {
    // eprintln!("one side");
    debug_assert!(s.len() == 3);
    // xd-re merőleges, kifelé mutat
    let xd_perp = xyd_perp.cross(&(s[2].diff - s[0].diff));
    // xd-n kívül van
    if xd_perp.dot(&-s[2].diff) > 0.0 {
        s.remove(1);
        return;
    }

    // yd-re merőleges, kifelé mutat
    let yd_perp = (s[2].diff - s[1].diff).cross(&xyd_perp);
    // yd-n kívül van
    if yd_perp.dot(&-s[2].diff) > 0.0 {
        s.remove(0);
    }
}

fn closest_point_to_contact(
    a: &impl Support,
    b: &impl Support,
    closest_point: &SupportPoint,
) -> Option<(Vec3, Vec3, Vec3)> {
    if closest_point.diff.magnitude() <= a.radius() + b.radius() {
        let normal = closest_point.diff.normalize();
        let b_point = closest_point.a - closest_point.diff;
        Some((
            closest_point.a - normal * a.radius(),
            b_point + normal * b.radius(),
            normal.normalize(),
        ))
    } else {
        None
    }
}

fn closest_simplex(s: &mut SimplexData) -> SupportPoint {
    best_simplex(s);
    match s.len() {
        0 => panic!("simplex has to contain at least 1 point"),
        1 => s[0],
        2 => {
            let ba = s[0] - s[1];
            let t = -s[1].diff.dot(&ba.diff) / ba.diff.magnitude_squared();
            debug_assert!(
                (0.0..=1.0).contains(&t),
                "invalid multiplier t = {t}"
            );
            s[1] + t * &ba
        }
        // 2 => closest_simplex_static::<2>(s),
        3 => closest_point_static::<3>(s),
        4 => closest_point_static::<4>(s),
        // 4 => SupportPoint {
        //     diff: Vec3::zeros(),
        //     a: Vec3::zeros(),
        // },
        _ => unreachable!(),
    }
}

fn closest_point_static<const N: usize>(s: &SimplexData) -> SupportPoint
where
    Const<N>: DimMin<Const<N>, Output = Const<N>>,
{
    let mut a = Matrix::zeros_generic(Const::<N>, Const::<N>);
    a.data.0[0][0] = 1.0;
    for i in 1..N {
        a.data.0[i][0] = 1.0f64;
        for j in 1..N {
            a.data.0[i][j] =
                (s[i].diff - s[0].diff).dot(&(s[j].diff - s[0].diff));
        }
    }
    let a_inverse = a.try_inverse().expect("a is invertible");
    let mut b = Matrix::zeros_generic(Const::<N>, Const::<1>);
    b[0] = 1.0;
    for i in 1..N {
        b[i] = -s[0].diff.dot(&(s[i].diff - s[0].diff));
    }
    let multipliers = a_inverse * b;
    multipliers
        .iter()
        .inspect(|m| {
            debug_assert!(
                m >= &&-TOLERANCE,
                "invalid multiplier m={m}, should be >= 0"
            );
        })
        .zip(s)
        .map(|(t, v)| *t * v)
        .reduce(|a, b| a + b)
        .unwrap()
}

pub fn epa(
    a: &impl Support,
    b: &impl Support,
    mut points: Vec<SupportPoint>,
) -> Option<(Vec3, Vec3, Vec3)> {
    debug_assert_eq!(points.len(), 4);
    let mut faces = vec![[0, 1, 2], [0, 2, 3], [0, 3, 1], [1, 2, 3]];
    let mut closest_points = vec![];
    let mut tmp = SimplexData::new();
    for [v1, v2, v3] in &faces {
        tmp.push(points[*v1]);
        tmp.push(points[*v2]);
        tmp.push(points[*v3]);
        let closest_point = closest_simplex(&mut tmp);
        tmp.clear();
        closest_points.push(closest_point);
    }
    let mut iter = 0;
    loop {
        let Some(minface) = closest_points
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                a.diff.magnitude().total_cmp(&b.diff.magnitude())
            })
            .map(|(i, _)| i)
        else {
            eprintln!("math has failed!");
            return None;
        };
        let new_point = SupportPoint::new(a, b, &closest_points[minface].diff);
        debug_assert!(
            new_point.diff.magnitude()
                >= closest_points[minface].diff.magnitude() - TOLERANCE,
        );
        if new_point.diff.dot(&closest_points[minface].diff)
            <= closest_points[minface].diff.magnitude_squared() + TOLERANCE
            || iter == EPA_MAX_ITER
            || points.iter().any(|p| p.diff == new_point.diff)
        {
            if iter == EPA_MAX_ITER {
                eprintln!("epa max reached");
            }
            // return handle_return_3(
            //     a,
            //     b,
            //     &points[faces[minface][0]],
            //     &points[faces[minface][0]],
            //     &points[faces[minface][0]],
            //     &closest_points[minface],
            // );
            let b_point =
                closest_points[minface].a - closest_points[minface].diff;
            return Some((
                closest_points[minface].a,
                b_point,
                -closest_points[minface].diff.normalize(),
            ));
        }
        let mut edges = vec![];
        let mut i = 0;
        debug_assert_eq!(faces.len(), closest_points.len());
        while i < faces.len() {
            if closest_points[i]
                .diff
                .dot(&(new_point.diff - closest_points[i].diff))
                >= 0.0
            {
                edges.add_or_remove(minmax(faces[i][0], faces[i][1]));
                edges.add_or_remove(minmax(faces[i][1], faces[i][2]));
                edges.add_or_remove(minmax(faces[i][2], faces[i][0]));
                faces.swap_remove(i);
                closest_points.swap_remove(i);
            } else {
                i += 1;
            }
        }
        debug_assert_eq!(faces.len(), closest_points.len());

        let mut new_faces = vec![];
        for (i, j) in edges {
            new_faces.push([i, j, points.len()]);
        }
        points.push(new_point);
        let mut new_closest_points = vec![];
        for [v1, v2, v3] in &new_faces {
            tmp.push(points[*v1]);
            tmp.push(points[*v2]);
            tmp.push(points[*v3]);
            let closest_point = closest_simplex(&mut tmp);
            tmp.clear();
            new_closest_points.push(closest_point);
        }
        faces.append(&mut new_faces);
        closest_points.append(&mut new_closest_points);
        iter += 1;
    }
}

impl Mul<&SupportPoint> for f64 {
    type Output = SupportPoint;

    fn mul(self, rhs: &SupportPoint) -> Self::Output {
        SupportPoint {
            diff: self * rhs.diff,
            a: self * rhs.a,
        }
    }
}

impl Add for SupportPoint {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.diff += rhs.diff;
        self.a += rhs.a;
        self
    }
}

impl Sub for SupportPoint {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self.diff -= rhs.diff;
        self.a -= rhs.a;
        self
    }
}

trait AddOrRemove<T: PartialEq> {
    fn add_or_remove(&mut self, elem: T);
}

impl<T: PartialEq> AddOrRemove<T> for Vec<T> {
    fn add_or_remove(&mut self, elem: T) {
        if let Some(index) = self.iter().position(|e| e == &elem) {
            self.swap_remove(index);
        } else {
            self.push(elem);
        }
    }
}

fn minmax<T: Ord>(a: T, b: T) -> (T, T) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

#[allow(unused)]
fn debug_simplex_data<'a>(s: impl IntoIterator<Item = &'a SupportPoint>) {
    for (i, p) in s.into_iter().enumerate() {
        eprintln!("A_{i} = ({}, {}, {})", p.diff.x, p.diff.y, p.diff.z);
    }
}
