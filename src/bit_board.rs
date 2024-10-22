use petgraph::graph::NodeIndex;
use std::collections::HashSet;


#[derive(Debug, PartialEq, Eq)]
pub struct BitBoard(u64);

impl BitBoard {
    pub fn from_node_indices(node_indices: HashSet<NodeIndex>) -> BitBoard {
        let mut result: u64 = 0;
        for node in node_indices {
            result += 1 << node.index();
        }
        return BitBoard(result)
    }

    pub fn attack_table(node_table: Vec<HashSet<NodeIndex>>) -> Vec<BitBoard> {
        let mut result: Vec<BitBoard> = vec![];
        for node_indices in node_table {
            result.push(BitBoard::from_node_indices(node_indices))
        }
        return result
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
    fn test_attack_table() {
        assert_eq!(
            BitBoard::attack_table(vec![
                HashSet::from_iter([
                    NodeIndex::new(0),
                    NodeIndex::new(1)
                ]),
                HashSet::from_iter([
                    NodeIndex::new(10),
                    NodeIndex::new(20)
                ])
            ]),
            vec![BitBoard(3), BitBoard(1049600)]
        )
    }
}
