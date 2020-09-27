use crate::{
    pane_grid::{Axis, Pane, Split},
    Rectangle, Size,
};

use std::collections::HashMap;

/// A layout node of a [`PaneGrid`].
///
/// [`PaneGrid`]: struct.PaneGrid.html
#[derive(Debug, Clone)]
pub enum Node {
    /// The region of this [`Node`] is split into two.
    ///
    /// [`Node`]: enum.Node.html
    Split {
        /// The [`Split`] of this [`Node`].
        ///
        /// [`Split`]: struct.Split.html
        /// [`Node`]: enum.Node.html
        id: Split,

        /// The direction of the split.
        axis: Axis,

        /// The ratio of the split in [0.0, 1.0].
        ratio: f32,

        /// The left/top [`Node`] of the split.
        ///
        /// [`Node`]: enum.Node.html
        a: Box<Node>,

        /// The right/bottom [`Node`] of the split.
        ///
        /// [`Node`]: enum.Node.html
        b: Box<Node>,
    },
    /// The region of this [`Node`] is taken by a [`Pane`].
    ///
    /// [`Pane`]: struct.Pane.html
    Pane(Pane),
}

impl Node {
    /// Returns an iterator over each [`Split`] in this [`Node`].
    ///
    /// [`Split`]: struct.Split.html
    /// [`Node`]: enum.Node.html
    pub fn splits(&self) -> impl Iterator<Item = &Split> {
        let mut unvisited_nodes = vec![self];

        std::iter::from_fn(move || {
            while let Some(node) = unvisited_nodes.pop() {
                match node {
                    Node::Split { id, a, b, .. } => {
                        unvisited_nodes.push(a);
                        unvisited_nodes.push(b);

                        return Some(id);
                    }
                    _ => {}
                }
            }

            None
        })
    }

    /// Returns the rectangular region for each [`Pane`] in the [`Node`] given
    /// the spacing between panes and the total available space.
    ///
    /// [`Pane`]: struct.Pane.html
    /// [`Node`]: enum.Node.html
    pub fn pane_regions(
        &self,
        spacing: f32,
        size: Size,
    ) -> HashMap<Pane, Rectangle> {
        let mut regions = HashMap::new();

        self.compute_regions(
            spacing,
            &Rectangle {
                x: 0.0,
                y: 0.0,
                width: size.width,
                height: size.height,
            },
            &mut regions,
        );

        regions
    }

    /// Returns the axis, rectangular region, and ratio for each [`Split`] in
    /// the [`Node`] given the spacing between panes and the total available
    /// space.
    ///
    /// [`Split`]: struct.Split.html
    /// [`Node`]: enum.Node.html
    pub fn split_regions(
        &self,
        spacing: f32,
        size: Size,
    ) -> HashMap<Split, (Axis, Rectangle, f32)> {
        let mut splits = HashMap::new();

        self.compute_splits(
            spacing,
            &Rectangle {
                x: 0.0,
                y: 0.0,
                width: size.width,
                height: size.height,
            },
            &mut splits,
        );

        splits
    }

    pub(crate) fn find(&mut self, pane: &Pane) -> Option<&mut Node> {
        match self {
            Node::Split { a, b, .. } => {
                a.find(pane).or_else(move || b.find(pane))
            }
            Node::Pane(p) => {
                if p == pane {
                    Some(self)
                } else {
                    None
                }
            }
        }
    }

    pub(crate) fn find_split(&mut self, split: &Split) -> Option<&mut Node> {
        match self {
            Node::Split { id, .. } if id == split => Some(self),
            Node::Split { a, b, .. } => {
                a.find_split(split).or_else(move || b.find_split(split))
            }
            Node::Pane(_) => None,
        }
    }

    pub(crate) fn split(&mut self, id: Split, axis: Axis, new_node: Node) {
        *self = Node::Split {
            id,
            axis,
            ratio: 0.5,
            a: Box::new(self.clone()),
            b: Box::new(new_node),
        };
    }

    pub(crate) fn update(&mut self, f: &impl Fn(&mut Node)) {
        match self {
            Node::Split { a, b, .. } => {
                a.update(f);
                b.update(f);
            }
            _ => {}
        }

        f(self);
    }

    pub(crate) fn resize(&mut self, split: &Split, percentage: f32) -> bool {
        match self {
            Node::Split {
                id, ratio, a, b, ..
            } => {
                if id == split {
                    *ratio = percentage;

                    true
                } else if a.resize(split, percentage) {
                    true
                } else {
                    b.resize(split, percentage)
                }
            }
            Node::Pane(_) => false,
        }
    }

    pub(crate) fn remove(&mut self, pane: &Pane) -> Option<Pane> {
        match self {
            Node::Split { a, b, .. } => {
                if a.pane() == Some(*pane) {
                    *self = *b.clone();
                    Some(self.first_pane())
                } else if b.pane() == Some(*pane) {
                    *self = *a.clone();
                    Some(self.first_pane())
                } else {
                    a.remove(pane).or_else(|| b.remove(pane))
                }
            }
            Node::Pane(_) => None,
        }
    }

    fn pane(&self) -> Option<Pane> {
        match self {
            Node::Split { .. } => None,
            Node::Pane(pane) => Some(*pane),
        }
    }

    fn first_pane(&self) -> Pane {
        match self {
            Node::Split { a, .. } => a.first_pane(),
            Node::Pane(pane) => *pane,
        }
    }

    fn compute_regions(
        &self,
        spacing: f32,
        current: &Rectangle,
        regions: &mut HashMap<Pane, Rectangle>,
    ) {
        match self {
            Node::Split {
                axis, ratio, a, b, ..
            } => {
                let (region_a, region_b) = axis.split(current, *ratio, spacing);

                a.compute_regions(spacing, &region_a, regions);
                b.compute_regions(spacing, &region_b, regions);
            }
            Node::Pane(pane) => {
                let _ = regions.insert(*pane, *current);
            }
        }
    }

    fn compute_splits(
        &self,
        spacing: f32,
        current: &Rectangle,
        splits: &mut HashMap<Split, (Axis, Rectangle, f32)>,
    ) {
        match self {
            Node::Split {
                axis,
                ratio,
                a,
                b,
                id,
            } => {
                let (region_a, region_b) = axis.split(current, *ratio, spacing);

                let _ = splits.insert(*id, (*axis, *current, *ratio));

                a.compute_splits(spacing, &region_a, splits);
                b.compute_splits(spacing, &region_b, splits);
            }
            Node::Pane(_) => {}
        }
    }
}

impl std::hash::Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Node::Split {
                id,
                axis,
                ratio,
                a,
                b,
            } => {
                id.hash(state);
                axis.hash(state);
                ((ratio * 100_000.0) as u32).hash(state);
                a.hash(state);
                b.hash(state);
            }
            Node::Pane(pane) => {
                pane.hash(state);
            }
        }
    }
}
