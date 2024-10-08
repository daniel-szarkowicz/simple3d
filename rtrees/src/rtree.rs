#![allow(clippy::new_without_default)]
#![allow(missing_debug_implementations)]

use crate::omt::AABB;
use std::fmt::Debug;

const NODE_MAX_CHILDREN: usize = 6;

pub struct RTree<T> {
    height: usize,
    root: Option<Node<T>>,
}

struct Node<T> {
    aabb: AABB,
    entry: Entry<T>,
}

struct Leaf<T> {
    aabb: AABB,
    data: T,
}

enum Entry<T> {
    Nodes(Vec<Node<T>>),
    Leaves(Vec<Leaf<T>>),
}

impl<T> RTree<T> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            height: 1,
            root: None,
        }
    }

    pub fn clear(&mut self) {
        self.root = None;
    }

    #[must_use]
    pub fn aabbs(&self) -> Vec<(usize, &AABB)> {
        let mut collector = vec![];
        if let Some(ref root) = self.root {
            root.aabbs_into(0, &mut collector);
        }
        collector
    }

    #[must_use]
    pub fn search(&self, aabb: &AABB) -> Vec<&T> {
        let mut collector = vec![];
        if let Some(ref root) = self.root {
            root.search_into(aabb, &mut collector);
        }
        collector
    }

    pub fn insert(&mut self, aabb: AABB, data: T) {
        self.root = Some(if let Some(mut root) = self.root.take() {
            if let InsertResult::Split(new_node) = root.insert(aabb, data) {
                let mut vec = Vec::with_capacity(NODE_MAX_CHILDREN + 1);
                let new_aabb = AABB::merge([&root.aabb, &new_node.aabb]);
                vec.push(root);
                vec.push(new_node);
                self.height += 1;
                Node {
                    aabb: new_aabb,
                    entry: Entry::Nodes(vec),
                }
            } else {
                root
            }
        } else {
            let mut vec = Vec::with_capacity(NODE_MAX_CHILDREN + 1);
            vec.push(Leaf { aabb, data });
            self.height += 1;
            Node {
                aabb,
                entry: Entry::Leaves(vec),
            }
        });
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

impl<T> Node<T> {
    fn search_into<'a>(&'a self, aabb: &AABB, collector: &mut Vec<&'a T>) {
        // let mut descends = 0;
        match self.entry {
            Entry::Nodes(ref nodes) => {
                for node in nodes {
                    if node.aabb.overlaps(aabb) {
                        node.search_into(aabb, collector);
                        // descends += 1;
                    }
                }
            }
            Entry::Leaves(ref leaves) => {
                for leaf in leaves {
                    if leaf.aabb.overlaps(aabb) {
                        collector.push(&leaf.data);
                        // descends += 1;
                    }
                }
            }
        }
        // println!("descends: {descends}");
    }

    fn insert(&mut self, aabb: AABB, data: T) -> InsertResult<T> {
        match self.entry {
            Entry::Nodes(ref mut nodes) => {
                self.aabb = AABB::merge([&self.aabb, &aabb]);
                if let InsertResult::Split(new_node) =
                    nodes.find_best_match(&aabb).insert(aabb, data)
                {
                    nodes.push(new_node);
                    if nodes.len() > NODE_MAX_CHILDREN {
                        let ((aabb1, nodes1), (aabb2, nodes2)) =
                            quadratic_split(nodes);
                        self.aabb = aabb1;
                        *nodes = nodes1;
                        InsertResult::Split(Self {
                            aabb: aabb2,
                            entry: Entry::Nodes(nodes2),
                        })
                    } else {
                        InsertResult::NoSplit
                    }
                } else {
                    InsertResult::NoSplit
                }
            }
            Entry::Leaves(ref mut leaves) => {
                self.aabb = AABB::merge([&self.aabb, &aabb]);
                leaves.push(Leaf { aabb, data });
                if leaves.len() > NODE_MAX_CHILDREN {
                    let ((aabb1, leaves1), (aabb2, leaves2)) = split(leaves);
                    self.aabb = aabb1;
                    *leaves = leaves1;
                    InsertResult::Split(Self {
                        aabb: aabb2,
                        entry: Entry::Leaves(leaves2),
                    })
                } else {
                    InsertResult::NoSplit
                }
            }
        }
    }

    fn aabbs_into<'a>(
        &'a self,
        depth: usize,
        collector: &mut Vec<(usize, &'a AABB)>,
    ) {
        collector.push((depth, &self.aabb));
        match self.entry {
            Entry::Nodes(ref nodes) => {
                for n in nodes {
                    n.aabbs_into(depth + 1, collector);
                }
            }
            Entry::Leaves(ref leaves) => {
                for l in leaves {
                    collector.push((depth + 1, &l.aabb));
                }
            }
        }
    }
}

#[allow(dead_code)]
/// Drains the nodes into two vectors using a heuristic to produce smaller AABBs
fn split<T: HasAABB>(nodes: &mut Vec<T>) -> ((AABB, Vec<T>), (AABB, Vec<T>)) {
    let (seed1, seed2) = nodes
        .iter()
        .enumerate()
        .flat_map(|(i, n1)| {
            nodes.iter().enumerate().skip(i + 1).map(move |(j, n2)| {
                (i, j, AABB::merge([n1.aabb(), n2.aabb()]).volume())
            })
        })
        .max_by(|(_, _, s1), (_, _, s2)| s1.total_cmp(s2))
        .map(|(i, j, _)| (i, j))
        .unwrap();
    let mut nodes1 = Vec::with_capacity(NODE_MAX_CHILDREN + 1);
    let mut nodes2 = Vec::with_capacity(NODE_MAX_CHILDREN + 1);
    // we need to worry about ordering
    assert!(seed1 < seed2);
    nodes2.push(nodes.swap_remove(seed2));
    nodes1.push(nodes.swap_remove(seed1));
    let mut aabb1 = nodes1[0].aabb().clone();
    let mut aabb2 = nodes2[0].aabb().clone();
    for node in nodes.drain(..) {
        let new_aabb1 = AABB::merge([&aabb1, node.aabb()]);
        let new_aabb2 = AABB::merge([&aabb2, node.aabb()]);
        let diff1 = new_aabb1.volume() - aabb1.volume();
        let diff2 = new_aabb2.volume() - aabb2.volume();
        if diff1 < diff2 {
            nodes1.push(node);
            aabb1 = new_aabb1;
        } else {
            nodes2.push(node);
            aabb2 = new_aabb2;
        }
    }
    ((aabb1, nodes1), (aabb2, nodes2))
}

#[allow(dead_code)]
fn quadratic_split<T: HasAABB>(
    nodes: &mut Vec<T>,
) -> ((AABB, Vec<T>), (AABB, Vec<T>)) {
    // PickSeeds for quadratic split as described in
    // https://infolab.usc.edu/csci599/Fall2001/paper/rstar-tree.pdf
    let (seed1, seed2) = nodes
        .iter()
        .enumerate()
        .flat_map(|(i, n1)| {
            nodes.iter().enumerate().skip(i + 1).map(move |(j, n2)| {
                (
                    i,
                    j,
                    AABB::merge([n1.aabb(), n2.aabb()]).volume()
                        - n1.aabb().volume()
                        - n2.aabb().volume(),
                )
            })
        })
        .max_by(|(_, _, s1), (_, _, s2)| s1.total_cmp(s2))
        .map(|(i, j, _)| (i, j))
        .unwrap();
    let mut nodes1 = Vec::with_capacity(NODE_MAX_CHILDREN + 1);
    let mut nodes2 = Vec::with_capacity(NODE_MAX_CHILDREN + 1);
    // we need to worry about ordering
    assert!(seed1 < seed2);
    nodes2.push(nodes.swap_remove(seed2));
    nodes1.push(nodes.swap_remove(seed1));
    let mut aabb1 = nodes1[0].aabb().clone();
    let mut aabb2 = nodes2[0].aabb().clone();
    while !nodes.is_empty() {
        // PickNext
        let next = nodes
            .iter()
            .enumerate()
            .map(|(i, n)| {
                (
                    i,
                    AABB::merge([&aabb1, n.aabb()]).volume()
                        - AABB::merge([&aabb2, n.aabb()]).volume().abs(),
                )
            })
            .max_by(|(_, diff1), (_, diff2)| diff1.total_cmp(diff2))
            .map(|(i, _)| i)
            .expect("nodes cannot be empty");
        let node = nodes.swap_remove(next);
        let new_aabb1 = AABB::merge([&aabb1, node.aabb()]);
        let new_aabb2 = AABB::merge([&aabb2, node.aabb()]);
        let diff1 = new_aabb1.volume() - aabb1.volume();
        let diff2 = new_aabb2.volume() - aabb2.volume();
        if diff1 < diff2 {
            nodes1.push(node);
            aabb1 = new_aabb1;
        } else {
            nodes2.push(node);
            aabb2 = new_aabb2;
        }
    }
    ((aabb1, nodes1), (aabb2, nodes2))
}

trait HasAABB {
    fn aabb(&self) -> &AABB;
}

impl<T> HasAABB for Node<T> {
    fn aabb(&self) -> &AABB {
        &self.aabb
    }
}

impl<T> HasAABB for Leaf<T> {
    fn aabb(&self) -> &AABB {
        &self.aabb
    }
}

trait FindBestMatch<T> {
    fn find_best_match(&mut self, aabb: &AABB) -> &mut Node<T>;
}

impl<T> FindBestMatch<T> for Vec<Node<T>> {
    fn find_best_match(&mut self, aabb: &AABB) -> &mut Node<T> {
        self.iter_mut()
            .map(|n| {
                (AABB::merge([&n.aabb, aabb]).volume() - n.aabb.volume(), n)
            })
            .min_by(|(s1, _), (s2, _)| s1.total_cmp(s2))
            .map(|(_, n)| n)
            .unwrap()
    }
}

#[must_use]
enum InsertResult<T> {
    Split(Node<T>),
    NoSplit,
}

impl<T> Debug for RTree<T>
where
    Node<T>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RTree {{ root: {:?} }}", self.root)
    }
}

impl<T> Debug for Node<T>
where
    Entry<T>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Node {{ aabb: {:?}, entry: {:?} }}",
            self.aabb, self.entry
        )
    }
}

impl<T> Debug for Leaf<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Leaf {{ aabb: {:?}, data: {:?} }}", self.aabb, self.data)
    }
}

impl<T> Debug for Entry<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nodes(v) => write!(f, "Nodes({v:?})"),
            Self::Leaves(v) => write!(f, "Leaf({v:?})"),
        }
    }
}
