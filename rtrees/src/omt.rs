use std::{any::Any, iter, ops::Range};

use rand::distributions::uniform::SampleRange;

#[derive(Debug)]
pub struct RTree<T> {
    layers: Vec<Vec<Node>>,
    leaves: Vec<Leaf<T>>,
}

#[derive(Debug)]
struct Node {
    aabb: AABB,
    start: usize,
    end: usize,
}

#[derive(Debug, Clone)]
pub struct Leaf<T> {
    pub aabb: AABB,
    pub data: T,
}

const MAX_NODE_SIZE: usize = 6;

impl<T> RTree<T> {
    pub fn new(mut leaves: Vec<Leaf<T>>) -> Self {
        let height = if leaves.len() <= MAX_NODE_SIZE {
            1
        } else {
            (leaves.len() - 1).ilog(MAX_NODE_SIZE) as usize + 1
        };
        let mut layers = Vec::with_capacity(height);
        layers.push(vec![Node {
            start: 0,
            end: leaves.len(),
            aabb: AABB::merge(leaves.iter().map(|l| &l.aabb)),
        }]);
        for _ in 1..height {
            let new_layer =
                Self::omt_new_layer(layers.last_mut().unwrap(), &mut leaves);
            layers.push(new_layer);
        }
        Self { layers, leaves }
    }

    fn omt_new_layer(
        old_layer: &mut [Node],
        leaves: &mut [Leaf<T>],
    ) -> Vec<Node> {
        let mut new_nodes = Vec::new();
        for node in old_layer {
            Self::omt_split(
                node,
                &mut leaves[node.start..node.end],
                &mut new_nodes,
            )
        }
        new_nodes
    }

    fn omt_split(
        node: &mut Node,
        node_leaves: &mut [Leaf<T>],
        new_nodes: &mut Vec<Node>,
    ) {
        let node_count = {
            let height = (node_leaves.len() as f64)
                .log(MAX_NODE_SIZE as f64)
                .ceil() as u32;
            (node_leaves.len() as f64 / MAX_NODE_SIZE.pow(height - 1) as f64)
                .ceil() as usize
        };
        let splits = calculate_splits(node_count, node_leaves.len(), 3);
        node_leaves.sort_unstable_by(|l1, l2| {
            l1.aabb.pos()[0].total_cmp(&l2.aabb.pos()[0])
        });
        let mut i = 0;
        for &size in &splits[0] {
            node_leaves[i..][..size].sort_unstable_by(|l1, l2| {
                l1.aabb.pos()[1].total_cmp(&l2.aabb.pos()[1])
            });
            i += size;
        }
        let mut i = 0;
        for &size in &splits[1] {
            node_leaves[i..][..size].sort_unstable_by(|l1, l2| {
                l1.aabb.pos()[2].total_cmp(&l2.aabb.pos()[2])
            });
            i += size;
        }

        let mut child_start = node.start;
        let mut i = 0;
        node.start = new_nodes.len();
        for &size in &splits[2] {
            new_nodes.push(Node {
                start: child_start,
                end: child_start + size,
                aabb: AABB::merge(
                    node_leaves[i..][..size].iter().map(|l| &l.aabb),
                ),
            });
            i += size;
            child_start += size
        }
        node.end = new_nodes.len();
    }

    pub fn aabbs(&self) -> AABBS<'_, T> {
        AABBS::new(self)
    }

    pub fn height(&self) -> usize {
        self.layers.len() + 1
    }

    pub fn query(&self, aabb: AABB) -> Query<'_, T> {
        Query::new(self, aabb)
    }

    pub fn leaves(&self) -> impl Iterator<Item = &Leaf<T>> {
        self.leaves.iter()
    }
}

pub struct AABBS<'a, T> {
    tree: &'a RTree<T>,
    level: usize,
    index: usize,
}

impl<'a, T> AABBS<'a, T> {
    pub fn new(tree: &'a RTree<T>) -> Self {
        Self {
            tree,
            level: 0,
            index: 0,
        }
    }
}

impl<'a, T> Iterator for AABBS<'a, T> {
    type Item = (&'a AABB, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(layer) = self.tree.layers.get(self.level) {
            if let Some(node) = layer.get(self.index) {
                self.index += 1;
                Some((&node.aabb, self.level))
            } else {
                self.level += 1;
                self.index = 0;
                self.next()
            }
        } else if let Some(leaf) = self.tree.leaves.get(self.index) {
            self.index += 1;
            Some((&leaf.aabb, self.level))
        } else {
            None
        }
    }
}

pub struct QueryItem<'rtree, T> {
    pub aabb: AABB,
    pub data: QueryData<'rtree, T>,
}

pub enum QueryData<'rtree, T> {
    Node { depth: usize },
    Leaf { data: &'rtree T },
}

impl<'rtree, T> QueryItem<'rtree, T> {
    /// Returns `true` if the query data is [`Node`].
    ///
    /// [`Node`]: QueryData::Node
    #[must_use]
    pub fn is_node(&self) -> bool {
        matches!(self.data, QueryData::Node { .. })
    }

    /// Returns `true` if the query data is [`Leaf`].
    ///
    /// [`Leaf`]: QueryData::Leaf
    #[must_use]
    pub fn is_leaf(&self) -> bool {
        matches!(self.data, QueryData::Leaf { .. })
    }
}

pub struct Query<'rtree, T> {
    tree: &'rtree RTree<T>,
    aabb: AABB,
    indices: Vec<(usize, usize)>,
    ret_root: bool,
}

impl<'rtree, T> Query<'rtree, T> {
    pub fn new(tree: &'rtree RTree<T>, aabb: AABB) -> Self {
        let height = tree.height();
        let mut indices = Vec::with_capacity(height - 1);
        let root = &tree.layers[0][0];
        indices.push((root.start, root.end));
        Self {
            tree,
            aabb,
            indices,
            ret_root: true,
        }
    }
}

impl<'rtree, T> Iterator for Query<'rtree, T> {
    type Item = QueryItem<'rtree, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ret_root {
            self.ret_root = false;
            return Some(QueryItem {
                aabb: self.tree.layers[0][0].aabb,
                data: QueryData::Node { depth: 0 },
            });
        }
        if self.indices.len() == self.tree.height() - 1 {
            // guess we doin leaves now
            let index = self.indices.last_mut().unwrap();
            while index.0 < index.1 {
                let leaf = &self.tree.leaves[index.0];
                index.0 += 1;
                if leaf.aabb.overlaps(&self.aabb) {
                    return Some(QueryItem {
                        aabb: leaf.aabb,
                        data: QueryData::Leaf { data: &leaf.data },
                    });
                }
            }
            // this node is done, go to next
            self.indices.pop();
            self.next()
        } else {
            let layer = self.indices.len();
            let Some(index) = self.indices.last_mut() else {
                // we finished iterating the tree
                return None;
            };
            while index.0 < index.1 {
                let node = &self.tree.layers[layer][index.0];
                index.0 += 1;
                if node.aabb.overlaps(&self.aabb) {
                    // we found a good node
                    self.indices.push((node.start, node.end));
                    return Some(QueryItem {
                        aabb: node.aabb,
                        data: QueryData::Node { depth: layer },
                    });
                }
            }
            // this node is done, go to next
            self.indices.pop();
            self.next()
        }
    }
}

fn calculate_splits(
    node_count: usize,
    item_count: usize,
    dimensions: usize,
) -> Vec<Vec<usize>> {
    let mut splits = Vec::with_capacity(dimensions + 1);
    splits.push(vec![1; item_count]);
    for i in 0..dimensions {
        let last = splits.last().unwrap();
        let chunk_count = (node_count as f64)
            .powf((dimensions - i) as f64 / dimensions as f64)
            .round() as usize;
        let item_count = last.len();
        let small_size = item_count / chunk_count;
        let large_size = small_size + 1;
        let large_count = item_count - chunk_count * small_size;
        let small_count = chunk_count - large_count;
        let mut i = 0;
        // TODO: distribute the items from `last` more evenly
        let new_split = iter::empty()
            .chain(iter::repeat(small_size).take(small_count))
            .chain(iter::repeat(large_size).take(large_count))
            .map(|c| {
                let res = last[i..][..c].iter().sum();
                i += c;
                res
            })
            .collect();
        splits.push(new_split);
    }
    splits.reverse();
    splits.pop();
    splits
}

pub fn rand_aabbs(
    n: usize,
    bounds: AABB,
    size_bounds: Range<f64>,
) -> Vec<AABB> {
    let mut aabbs = Vec::with_capacity(n);
    for _ in 0..n {
        aabbs.push(rand_aabb(bounds, size_bounds.clone()));
    }
    aabbs
}

pub fn rand_aabb(bounds: AABB, size_bounds: Range<f64>) -> AABB {
    let mut rng = rand::thread_rng();
    let half_size = [
        size_bounds.clone().sample_single(&mut rng) / 0.5,
        size_bounds.clone().sample_single(&mut rng) / 0.5,
        size_bounds.clone().sample_single(&mut rng) / 0.5,
    ];
    let pos = [
        (bounds.min[0] + half_size[0]..bounds.max[0] - half_size[0])
            .sample_single(&mut rng),
        (bounds.min[1] + half_size[1]..bounds.max[1] - half_size[1])
            .sample_single(&mut rng),
        (bounds.min[2] + half_size[2]..bounds.max[2] - half_size[2])
            .sample_single(&mut rng),
    ];
    AABB {
        min: [0, 1, 2].map(|i| pos[i] - half_size[i]),
        max: [0, 1, 2].map(|i| pos[i] + half_size[i]),
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

impl AABB {
    pub fn pos(&self) -> [f64; 3] {
        [0, 1, 2].map(|i| (self.min[i] + self.max[i]) / 2.0)
    }

    pub fn size(&self) -> [f64; 3] {
        [0, 1, 2].map(|i| self.max[i] - self.min[i])
    }

    pub fn merge<'a>(aabbs: impl IntoIterator<Item = &'a AABB>) -> AABB {
        let mut iter = aabbs.into_iter();
        let Some(first) = iter.next() else {
            return AABB {
                min: [std::f64::MAX; 3],
                max: [std::f64::MIN; 3],
            };
        };
        iter.fold(*first, |mut a, b| {
            a.min[0] = a.min[0].min(b.min[0]);
            a.min[1] = a.min[1].min(b.min[1]);
            a.min[2] = a.min[2].min(b.min[2]);
            a.max[0] = a.max[0].max(b.max[0]);
            a.max[1] = a.max[1].max(b.max[1]);
            a.max[2] = a.max[2].max(b.max[2]);
            a
        })
    }

    pub fn volume(&self) -> f64 {
        let [w, h, d] = self.size();
        w * h * d
    }

    pub fn overlaps(&self, other: &AABB) -> bool {
        self.min[0] <= other.max[0]
            && self.min[1] <= other.max[1]
            && self.min[2] <= other.max[2]
            && other.min[0] <= self.max[0]
            && other.min[1] <= self.max[1]
            && other.min[2] <= self.max[2]
    }
}

impl<T> Leaf<T> {
    pub fn new(aabb: AABB, data: T) -> Self {
        Self { aabb, data }
    }
}

impl Leaf<()> {
    pub fn new_empty(aabb: AABB) -> Self {
        Self::new(aabb, ())
    }
}
