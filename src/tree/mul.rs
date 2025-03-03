use std::hash::Hash;

use super::fmt::PrintTree;
use qudit_core::HasPeriods;
use qudit_core::HasParams;
use qudit_core::RealScalar;
use qudit_core::QuditRadices;
use qudit_core::QuditSystem;
use super::tree::ExpressionTree;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct MulNode {
    pub left: Box<ExpressionTree>,
    pub right: Box<ExpressionTree>,
    left_params: usize,
    right_params: usize,
    dimension: usize,
}

impl MulNode {
    pub fn new(left: ExpressionTree, right: ExpressionTree) -> MulNode {
        if right.radices() != left.radices() {
            panic!("Left and right node do not have same dimension in multiply node.");
        }

        let left_params = left.num_params();
        let right_params = right.num_params();
        let _left_radices = left.radices();
        let _right_radices = right.radices();
        let dimension = left.dimension();

        MulNode {
            left: Box::new(left),
            right: Box::new(right),
            left_params,
            right_params,
            dimension,
        }
    }
}

impl HasParams for MulNode {
    fn num_params(&self) -> usize {
        self.left_params + self.right_params
    }
}

impl<R: RealScalar> HasPeriods<R> for MulNode {
    fn periods(&self) -> Vec<std::ops::Range<R>> {
        self.left
            .periods()
            .iter()
            .chain(self.right.periods().iter())
            .cloned()
            .collect()
    }
}

impl QuditSystem for MulNode {
    fn radices(&self) -> QuditRadices {
        self.left.radices()
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

impl PrintTree for MulNode {
    fn write_tree(&self, prefix: &str, fmt: &mut std::fmt::Formatter<'_>) {
        writeln!(fmt, "{}Mul", prefix).unwrap();
        let left_prefix = self.modify_prefix_for_child(prefix, false);
        let right_prefix = self.modify_prefix_for_child(prefix, true);
        self.left.write_tree(&left_prefix, fmt);
        self.right.write_tree(&right_prefix, fmt);
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::math::UnitaryBuilder;
//     use crate::sim::node::strategies::{arbitrary_nodes, nodes_and_params};
//     use crate::sim::node::Node;
//     use proptest::prelude::*;

//     //TODO: Build strategy for generating nodes with same radices; redo tests

//     proptest! {
//         #[test]
//         fn does_not_crash(node in arbitrary_nodes()) {
//             let _ = Node::Mul(MulNode::new(node.clone(), node));
//         }

//         // #[test]
//         // fn unitary_equals_unitary_builder((mut node, params) in
// nodes_and_params()) {         //     let mut mul_node =
// Node::Mul(MulNode::new(node.clone(), node.clone()));         //     let
// radices = mul_node.get_radices();         //     let mul_utry =
// mul_node.get_unitary_ref(&[params.clone(), params.clone()].concat());

//         //     let mut builder = UnitaryBuilder::new(radices);
//         //     let loc = Vec::from_iter(0..node.get_num_qudits());
//         //     builder.apply_right(node.get_unitary_ref(&params).view(),
// &loc, false);         //
// builder.apply_right(node.get_unitary_ref(&params).view(), &loc, false);
//         //     let utry = builder.get_unitary();
//         //     assert!((mul_utry - utry).opnorm_fro().unwrap() < 1e-8);
//         // }

//         #[test]
//         fn num_params_equals_sum_nodes(node in arbitrary_nodes())
//         {
//             let mul_node = Node::Mul(MulNode::new(node.clone(),
// node.clone()));             let num_params = mul_node.get_num_params();
//             assert_eq!(num_params, node.get_num_params() +
// node.get_num_params());         }

//         #[test]
//         fn radices_equals_same_radices(node in arbitrary_nodes())
//         {
//             let mul_node = Node::Mul(MulNode::new(node.clone(),
// node.clone()));             let radices = mul_node.get_radices();
//             assert_eq!(radices, node.get_radices());
//         }

//         #[test]
//         fn dimension_equals_same_dimension(node in arbitrary_nodes())
//         {
//             let mul_node = Node::Mul(MulNode::new(node.clone(),
// node.clone()));             let radices = mul_node.get_dimension();
//             assert_eq!(radices, node.get_dimension());
//         }

//         #[test]
//         fn is_hashable(node in arbitrary_nodes()) {
//             let mul_node = Node::Mul(MulNode::new(node.clone(),
// node.clone()));             let mut hasher =
// std::collections::hash_map::DefaultHasher::new();
// mul_node.hash(&mut hasher);             let _ = hasher.finish();
//         }

//         #[test]
//         fn is_hashable_set_insert(node in arbitrary_nodes()) {
//             let mut set = std::collections::HashSet::new();
//             let mul_node = Node::Mul(MulNode::new(node.clone(),
// node.clone()));             set.insert(mul_node.clone());
//             assert!(set.contains(&mul_node));
//         }

//         #[test]
//         fn equals_have_equal_hashes(node in arbitrary_nodes()) {
//             let mul_node1 = Node::Mul(MulNode::new(node.clone(),
// node.clone()));             let mul_node2 =
// Node::Mul(MulNode::new(node.clone(), node.clone()));
// assert_eq!(mul_node1, mul_node2);             let mut hasher1 =
// std::collections::hash_map::DefaultHasher::new();             let mut hasher2
// = std::collections::hash_map::DefaultHasher::new();
// mul_node1.hash(&mut hasher1);             mul_node2.hash(&mut hasher2);
//             assert_eq!(hasher1.finish(), hasher2.finish());
//         }

//         // TODO: Implement gradient tests with circuit.get_gradient
//     }
// }
