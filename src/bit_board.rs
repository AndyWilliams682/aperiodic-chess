use petgraph::graph::NodeIndex;
use std::collections::HashSet;
use std::ops::{Sub, BitAnd};


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BitBoard(u64);

impl BitBoard {
    pub fn from_node_indices(node_indices: HashSet<NodeIndex>) -> BitBoard {
        let mut result: u64 = 0;
        for node in node_indices {
            result += 1 << node.index();
        }
        return BitBoard(result)
    }
}

impl Sub for BitBoard {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        BitBoard(
            (self.0 | !other.0) + 1
        )
    }
}

impl BitAnd for BitBoard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        BitBoard(
            self.0 & rhs.0
        )
    }
}

pub struct CarryRipple {
    mask: BitBoard,
    current_subset: BitBoard,
}

impl Iterator for CarryRipple {
    type Item = BitBoard;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_subset == self.mask {
            return None
        }
        self.current_subset = (self.current_subset - self.mask) & self.mask;
        Some(self.current_subset)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        assert_eq!(
            BitBoard::from_node_indices(HashSet::from_iter([NodeIndex::new(0), NodeIndex::new(25)])),
            BitBoard(33554433)
        )
    }

    #[test]
    fn test_carry_ripple() {
        let mut test = CarryRipple { 
            mask: BitBoard(3), current_subset: BitBoard(0)
        };
        assert_eq!(
            test.next().unwrap(),
            BitBoard(1)
        );
        assert_eq!(
            test.next().unwrap(),
            BitBoard(2)
        );
        assert_eq!(
            test.next().unwrap(),
            BitBoard(3)
        );
        assert_eq!(
            test.next(),
            None
        )
    }
}
